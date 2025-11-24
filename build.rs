// build.rs - Script chạy trước khi compile
// Để thêm icon và metadata cho file .exe trên Windows

#[cfg(windows)]
extern crate winres;

fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        
        // Đặt icon cho .exe file (cần có file icon.ico trong assets/)
        res.set_icon("assets/icon.ico");
        
        // Thêm metadata
        res.set("ProductName", "Advanced RAM Manager");
        res.set("FileDescription", "RAM Management Tool for Windows");
        res.set("CompanyName", "Your Name");
        res.set("LegalCopyright", "Copyright © 2024");
        res.set("ProductVersion", "1.0.0");
        res.set("FileVersion", "1.0.0");
        
        // Require admin privileges
        res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
"#);
        
        res.compile().unwrap();
    }
}