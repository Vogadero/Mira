// build.rs - 构建脚本，生成构建时信息

use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 生成构建时间和 Git 信息
    EmitBuilder::builder()
        .build_timestamp()
        .git_sha(false)
        .emit()?;
    
    // Windows 特定配置：隐藏控制台窗口
    #[cfg(target_os = "windows")]
    {
        // 检查是否设置了 MIRA_SHOW_CONSOLE 环境变量
        let show_console = std::env::var("MIRA_SHOW_CONSOLE")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase();
        
        // 在 release 模式下默认隐藏控制台
        // 在 debug 模式下默认显示控制台
        let is_release = std::env::var("PROFILE")
            .map(|p| p == "release")
            .unwrap_or(false);
        
        // 如果是 release 模式且没有明确要求显示控制台，则隐藏
        if is_release && show_console != "true" && show_console != "1" {
            // 使用 winres 设置 Windows 子系统为 windows（隐藏控制台）
            let mut res = winres::WindowsResource::new();
            res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity
    version="1.0.0.0"
    processorArchitecture="*"
    name="Mira"
    type="win32"
  />
  <description>Mira Desktop Camera</description>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
"#);
            
            if let Err(e) = res.compile() {
                eprintln!("Warning: Failed to compile Windows resources: {}", e);
            }
            
            // 设置链接器参数以隐藏控制台
            println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
            println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
            
            println!("cargo:warning=Building with hidden console (release mode)");
        } else {
            println!("cargo:warning=Building with visible console (debug mode or MIRA_SHOW_CONSOLE=true)");
        }
    }
    
    println!("cargo:rerun-if-env-changed=MIRA_SHOW_CONSOLE");
    println!("cargo:rerun-if-env-changed=PROFILE");
    
    Ok(())
}