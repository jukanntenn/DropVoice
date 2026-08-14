//! 领域模型（纯数据，无 IO）。

pub mod device;

// 重新导出所有公共类型，便于 `use crate::domain::*`。
pub use device::{
    Device, DeviceAddress, DeviceRegisterRequest, DeviceRegisterResponse, DeviceStatus, Platform,
    StatusReportRequest, StatusReportResponse,
};
