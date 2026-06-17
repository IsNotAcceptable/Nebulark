use std::path::Path;

const POLICY_PATH: &str = "/usr/share/polkit-1/actions/org.nebulark.policy";

pub fn ensure_polkit_policy(exe: &Path) -> anyhow::Result<()> {
    if Path::new(POLICY_PATH).exists() {
        let content = std::fs::read_to_string(POLICY_PATH).unwrap_or_default();
        let exe_str = exe.to_string_lossy();
        if content.contains(exe_str.as_ref()) {
            return Ok(());
        }
    }

    let policy = generate_policy(exe);
    let tmp = std::env::temp_dir().join("nebulark.policy.tmp");
    std::fs::write(&tmp, &policy)?;
    let status = std::process::Command::new("pkexec")
        .args([
            "sh",
            "-c",
            &format!(
                "cp '{}' '{}' && chmod 644 '{}'",
                tmp.display(),
                POLICY_PATH,
                POLICY_PATH
            ),
        ])
        .status()?;

    let _ = std::fs::remove_file(&tmp);

    if !status.success() {
        anyhow::bail!("Failed to install polkit policy");
    }

    Ok(())
}

fn generate_policy(exe: &Path) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE policyconfig PUBLIC
 "-//freedesktop//DTD PolicyKit Policy Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/PolicyKit/1/policyconfig.dtd">
<policyconfig>
  <action id="org.nebulark.daemon">
    <description>Run Nebulark VPN daemon</description>
    <message>Authentication required to manage VPN tunnel</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin_keep</allow_active>
    </defaults>
    <annotate key="org.freedesktop.policykit.exec.path">{}</annotate>
    <annotate key="org.freedesktop.policykit.exec.allow_gui">true</annotate>
  </action>
</policyconfig>
"#,
        exe.display()
    )
}
