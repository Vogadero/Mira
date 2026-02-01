// 窗口管理模块

pub mod manager;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod scaling_tests;

pub use manager::WindowManager;
