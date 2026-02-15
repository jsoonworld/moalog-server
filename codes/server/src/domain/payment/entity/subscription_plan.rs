use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 구독 플랜 타입
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "PlanName")]
pub enum PlanName {
    #[sea_orm(string_value = "FREE")]
    Free,
    #[sea_orm(string_value = "PRO")]
    Pro,
    #[sea_orm(string_value = "TEAM")]
    Team,
}

impl PlanName {
    /// 월간 AI 분석 제한 횟수 (None = 무제한)
    pub fn ai_analysis_limit(&self) -> Option<i32> {
        match self {
            PlanName::Free => Some(3),
            PlanName::Pro | PlanName::Team => None,
        }
    }

    /// 월간 AI 어시스턴트 제한 횟수 (None = 무제한)
    pub fn ai_assistant_limit(&self) -> Option<i32> {
        match self {
            PlanName::Free => Some(3),
            PlanName::Pro | PlanName::Team => None,
        }
    }

    /// PDF 내보내기 기능 사용 가능 여부
    pub fn pdf_export_enabled(&self) -> bool {
        match self {
            PlanName::Free => false,
            PlanName::Pro | PlanName::Team => true,
        }
    }

    /// 회고방 최대 인원 (-1 = 무제한)
    pub fn max_room_members(&self) -> i32 {
        match self {
            PlanName::Free | PlanName::Pro => 10,
            PlanName::Team => -1,
        }
    }

    /// 월간 가격 (원 단위)
    pub fn monthly_price(&self) -> i32 {
        match self {
            PlanName::Free => 0,
            PlanName::Pro => 9_900,
            PlanName::Team => 29_900,
        }
    }

    /// 표시 이름
    pub fn display_name(&self) -> &'static str {
        match self {
            PlanName::Free => "Free",
            PlanName::Pro => "Pro",
            PlanName::Team => "Team",
        }
    }

    /// 문자열에서 PlanName 파싱
    pub fn from_str_value(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "FREE" => Some(PlanName::Free),
            "PRO" => Some(PlanName::Pro),
            "TEAM" => Some(PlanName::Team),
            _ => None,
        }
    }
}
