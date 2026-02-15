use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

use crate::utils::error::AppError;

// ─── FluxPay API DTOs ──────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FluxPayCreateOrderRequest {
    pub user_id: String,
    pub line_items: Vec<FluxPayLineItem>,
    pub currency: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FluxPayLineItem {
    pub product_id: String,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FluxPayCreatePaymentRequest {
    pub order_id: String,
    pub amount: i64,
    pub currency: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct FluxPayOrderResponse {
    pub success: bool,
    pub data: Option<FluxPayOrderData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct FluxPayOrderData {
    pub id: String,
    pub status: String,
    pub total_amount: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct FluxPayPaymentResponse {
    pub success: bool,
    pub data: Option<FluxPayPaymentData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct FluxPayPaymentData {
    pub id: String,
    pub order_id: String,
    pub status: String,
    pub amount: f64,
}

// ─── FluxPay HTTP Client ──────────────────────────────────────

/// FluxPay 결제 엔진 HTTP 클라이언트
#[derive(Clone)]
pub struct FluxPayClient {
    client: Client,
    base_url: String,
    enabled: bool,
}

impl FluxPayClient {
    pub fn new(base_url: String, enabled: bool) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build reqwest client for FluxPay");
        Self {
            client,
            base_url,
            enabled,
        }
    }

    /// FluxPay에 주문 생성
    pub async fn create_order(
        &self,
        req: FluxPayCreateOrderRequest,
    ) -> Result<FluxPayOrderData, AppError> {
        if !self.enabled {
            warn!("FluxPay 클라이언트가 비활성화 상태입니다.");
            return Err(AppError::FluxPayServiceError(
                "FluxPay 서비스가 비활성화 상태입니다.".to_string(),
            ));
        }

        let url = format!("{}/api/v1/orders", self.base_url);
        debug!(url = %url, "FluxPay 주문 생성 요청");

        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                error!("FluxPay 주문 생성 연결 실패: {}", e);
                AppError::FluxPayServiceError(format!("FluxPay 연결 실패: {}", e))
            })?;

        let body: FluxPayOrderResponse = resp.json().await.map_err(|e| {
            error!("FluxPay 주문 응답 파싱 실패: {}", e);
            AppError::FluxPayServiceError(format!("FluxPay 응답 파싱 실패: {}", e))
        })?;

        body.data.ok_or_else(|| {
            AppError::PaymentFailed("FluxPay 주문 생성에 실패했습니다.".to_string())
        })
    }

    /// FluxPay에 결제 생성
    pub async fn create_payment(
        &self,
        req: FluxPayCreatePaymentRequest,
    ) -> Result<FluxPayPaymentData, AppError> {
        if !self.enabled {
            warn!("FluxPay 클라이언트가 비활성화 상태입니다.");
            return Err(AppError::FluxPayServiceError(
                "FluxPay 서비스가 비활성화 상태입니다.".to_string(),
            ));
        }

        let url = format!("{}/api/v1/payments", self.base_url);
        debug!(url = %url, "FluxPay 결제 생성 요청");

        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                error!("FluxPay 결제 생성 연결 실패: {}", e);
                AppError::FluxPayServiceError(format!("FluxPay 연결 실패: {}", e))
            })?;

        let body: FluxPayPaymentResponse = resp.json().await.map_err(|e| {
            error!("FluxPay 결제 응답 파싱 실패: {}", e);
            AppError::FluxPayServiceError(format!("FluxPay 응답 파싱 실패: {}", e))
        })?;

        body.data.ok_or_else(|| {
            AppError::PaymentFailed("FluxPay 결제 생성에 실패했습니다.".to_string())
        })
    }

    /// FluxPay 결제 상태 조회
    #[allow(dead_code)]
    pub async fn get_payment(&self, payment_id: &str) -> Result<FluxPayPaymentData, AppError> {
        if !self.enabled {
            return Err(AppError::FluxPayServiceError(
                "FluxPay 서비스가 비활성화 상태입니다.".to_string(),
            ));
        }

        let url = format!("{}/api/v1/payments/{}", self.base_url, payment_id);
        debug!(url = %url, "FluxPay 결제 조회 요청");

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                error!("FluxPay 결제 조회 연결 실패: {}", e);
                AppError::FluxPayServiceError(format!("FluxPay 연결 실패: {}", e))
            })?;

        let body: FluxPayPaymentResponse = resp.json().await.map_err(|e| {
            error!("FluxPay 결제 응답 파싱 실패: {}", e);
            AppError::FluxPayServiceError(format!("FluxPay 응답 파싱 실패: {}", e))
        })?;

        body.data.ok_or_else(|| {
            AppError::PaymentFailed("FluxPay 결제 조회에 실패했습니다.".to_string())
        })
    }
}
