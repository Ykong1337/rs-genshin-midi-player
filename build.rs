use std::io;

#[cfg(target_os = "windows")]
use winres;

#[cfg(target_os = "windows")]
fn main() -> io::Result<()> {
    let mut res = winres::WindowsResource::new();
    res.set_manifest(
        r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#,
    );
    res.set_icon("resources/lyre.ico");
    res.compile()?;
    Ok(())
}
