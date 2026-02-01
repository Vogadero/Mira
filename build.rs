// build.rs - 构建脚本，生成构建时信息

use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 生成构建时间和 Git 信息
    EmitBuilder::builder()
        .build_timestamp()
        .git_sha(false)
        .emit()?;
    
    Ok(())
}