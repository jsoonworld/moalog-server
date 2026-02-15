use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::{error, info, warn};

use crate::state::AppState;
use crate::utils::error::AppError;

use super::dto::{FluxPayWebhookPayload, PlanResponse, SubscriptionResponse};
use super::entity::member_subscription::{self, SubscriptionStatus};
use super::entity::subscription_plan::PlanName;
use super::fluxpay_client::{
    FluxPayCreateOrderRequest, FluxPayCreatePaymentRequest, FluxPayLineItem,
};

pub struct PaymentService;

impl PaymentService {
    /// 플랜 목록 조회 (정적 데이터)
    pub fn list_plans() -> Vec<PlanResponse> {
        vec![
            PlanResponse::from_plan(PlanName::Free),
            PlanResponse::from_plan(PlanName::Pro),
            PlanResponse::from_plan(PlanName::Team),
        ]
    }

    /// 현재 구독 조회
    pub async fn get_subscription(
        state: &AppState,
        member_id: i64,
    ) -> Result<SubscriptionResponse, AppError> {
        let subscription = member_subscription::Entity::find()
            .filter(member_subscription::Column::MemberId.eq(member_id))
            .filter(member_subscription::Column::Status.eq(SubscriptionStatus::Active))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        match subscription {
            Some(sub) => Ok(Self::to_subscription_response(&sub)),
            None => {
                // 구독이 없으면 기본 FREE 플랜 반환
                Ok(SubscriptionResponse {
                    subscription_id: 0,
                    plan_name: PlanName::Free,
                    status: SubscriptionStatus::Active,
                    started_at: "".to_string(),
                    expires_at: None,
                    cancelled_at: None,
                })
            }
        }
    }

    /// 구독 생성 (FluxPay 연동)
    pub async fn create_subscription(
        state: &AppState,
        member_id: i64,
        plan_name_str: &str,
    ) -> Result<SubscriptionResponse, AppError> {
        // 1. 플랜 이름 검증
        let plan = PlanName::from_str_value(plan_name_str)
            .ok_or_else(|| AppError::InvalidPlan("유효하지 않은 플랜입니다.".to_string()))?;

        // 2. 기존 활성 구독 확인
        let existing = member_subscription::Entity::find()
            .filter(member_subscription::Column::MemberId.eq(member_id))
            .filter(member_subscription::Column::Status.eq(SubscriptionStatus::Active))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if let Some(existing_sub) = existing {
            if existing_sub.plan_name == plan {
                return Err(AppError::SubscriptionAlreadyActive(
                    "이미 동일한 플랜을 구독 중입니다.".to_string(),
                ));
            }
            // 기존 구독 해지 후 새 구독 생성
            let mut active_model: member_subscription::ActiveModel = existing_sub.into();
            active_model.status = Set(SubscriptionStatus::Cancelled);
            active_model.cancelled_at = Set(Some(Utc::now().naive_utc()));
            active_model.updated_at = Set(Utc::now().naive_utc());
            active_model
                .update(&state.db)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;
        }

        // 3. FREE 플랜은 즉시 활성화
        if plan == PlanName::Free {
            let new_subscription = member_subscription::ActiveModel {
                member_id: Set(member_id),
                plan_name: Set(PlanName::Free),
                status: Set(SubscriptionStatus::Active),
                started_at: Set(Utc::now().naive_utc()),
                created_at: Set(Utc::now().naive_utc()),
                updated_at: Set(Utc::now().naive_utc()),
                ..Default::default()
            };
            let inserted = new_subscription
                .insert(&state.db)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;

            info!(member_id = member_id, plan = "FREE", "Free 구독 활성화 완료");
            return Ok(Self::to_subscription_response(&inserted));
        }

        // 4. 유료 플랜: FluxPay 주문/결제 생성
        let price = plan.monthly_price() as i64;
        let plan_display = plan.display_name().to_string();

        // FluxPay 주문 생성
        let order = state
            .fluxpay_client
            .create_order(FluxPayCreateOrderRequest {
                user_id: member_id.to_string(),
                line_items: vec![FluxPayLineItem {
                    product_id: format!("plan_{}", plan_display.to_lowercase()),
                    product_name: format!("Moalog {} 플랜 (월간)", plan_display),
                    quantity: 1,
                    unit_price: price,
                }],
                currency: "KRW".to_string(),
            })
            .await?;

        // FluxPay 결제 생성
        let payment = state
            .fluxpay_client
            .create_payment(FluxPayCreatePaymentRequest {
                order_id: order.id.clone(),
                amount: price,
                currency: "KRW".to_string(),
            })
            .await?;

        // 5. DB에 구독 저장 (PENDING 상태 — 결제 완료 후 webhook으로 ACTIVE)
        let new_subscription = member_subscription::ActiveModel {
            member_id: Set(member_id),
            plan_name: Set(plan),
            status: Set(SubscriptionStatus::Pending),
            fluxpay_order_id: Set(Some(order.id)),
            fluxpay_payment_id: Set(Some(payment.id)),
            started_at: Set(Utc::now().naive_utc()),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };
        let inserted = new_subscription
            .insert(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        info!(
            member_id = member_id,
            plan = %plan_display,
            "구독 생성 완료 (결제 대기 중)"
        );
        Ok(Self::to_subscription_response(&inserted))
    }

    /// 구독 해지
    pub async fn cancel_subscription(
        state: &AppState,
        member_id: i64,
    ) -> Result<(), AppError> {
        let subscription = member_subscription::Entity::find()
            .filter(member_subscription::Column::MemberId.eq(member_id))
            .filter(member_subscription::Column::Status.eq(SubscriptionStatus::Active))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let sub = subscription.ok_or_else(|| {
            AppError::SubscriptionNotFound("활성 구독이 없습니다.".to_string())
        })?;

        if sub.plan_name == PlanName::Free {
            return Err(AppError::InvalidPlan(
                "Free 플랜은 해지할 수 없습니다.".to_string(),
            ));
        }

        let mut active_model: member_subscription::ActiveModel = sub.into();
        active_model.status = Set(SubscriptionStatus::Cancelled);
        active_model.cancelled_at = Set(Some(Utc::now().naive_utc()));
        active_model.updated_at = Set(Utc::now().naive_utc());
        active_model
            .update(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        info!(member_id = member_id, "구독 해지 완료");
        Ok(())
    }

    /// FluxPay 웹훅 처리
    pub async fn handle_fluxpay_webhook(
        state: &AppState,
        payload: FluxPayWebhookPayload,
    ) -> Result<(), AppError> {
        info!(
            event_type = %payload.event_type,
            order_id = %payload.order_id,
            payment_id = %payload.payment_id,
            status = %payload.status,
            "FluxPay 웹훅 수신"
        );

        // order_id로 구독 조회
        let subscription = member_subscription::Entity::find()
            .filter(member_subscription::Column::FluxpayOrderId.eq(&payload.order_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let sub = match subscription {
            Some(s) => s,
            None => {
                warn!(
                    order_id = %payload.order_id,
                    "웹훅: 해당 주문 ID의 구독을 찾을 수 없음"
                );
                return Ok(());
            }
        };

        let mut active_model: member_subscription::ActiveModel = sub.into();
        active_model.updated_at = Set(Utc::now().naive_utc());

        match payload.event_type.as_str() {
            "payment.confirmed" => {
                active_model.status = Set(SubscriptionStatus::Active);
                active_model.started_at = Set(Utc::now().naive_utc());
                info!(
                    order_id = %payload.order_id,
                    "구독 활성화 완료 (결제 확인)"
                );
            }
            "payment.failed" => {
                active_model.status = Set(SubscriptionStatus::Expired);
                error!(
                    order_id = %payload.order_id,
                    "결제 실패로 구독 만료 처리"
                );
            }
            _ => {
                warn!(
                    event_type = %payload.event_type,
                    "알 수 없는 웹훅 이벤트 타입"
                );
                return Ok(());
            }
        }

        active_model
            .update(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(())
    }

    /// 회원의 현재 플랜 조회 (AI 제한 체크용)
    pub async fn get_member_plan(
        state: &AppState,
        member_id: i64,
    ) -> Result<PlanName, AppError> {
        let subscription = member_subscription::Entity::find()
            .filter(member_subscription::Column::MemberId.eq(member_id))
            .filter(member_subscription::Column::Status.eq(SubscriptionStatus::Active))
            .one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(subscription
            .map(|s| s.plan_name)
            .unwrap_or(PlanName::Free))
    }

    // ─── 헬퍼 ──────────────────────────────────────────────────

    fn to_subscription_response(sub: &member_subscription::Model) -> SubscriptionResponse {
        SubscriptionResponse {
            subscription_id: sub.subscription_id,
            plan_name: sub.plan_name,
            status: sub.status,
            started_at: sub.started_at.to_string(),
            expires_at: sub.expires_at.map(|d| d.to_string()),
            cancelled_at: sub.cancelled_at.map(|d| d.to_string()),
        }
    }
}
