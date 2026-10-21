use std::process::Command;

pub struct ServiceManager;

impl ServiceManager {
    pub const SERVICE_NAME: &'static str = "ApexConnectService";
    pub const DISPLAY_NAME: &'static str = "ApexConnect Unattended Remote Access Service";

    /// Install Windows background service via sc.exe
    pub fn install(exe_path: &str) -> Result<String, String> {
        let bin_path_arg = format!("\"{}\" run-service", exe_path);
        let output = Command::new("sc.exe")
            .args([
                "create",
                Self::SERVICE_NAME,
                &format!("binPath= {}", bin_path_arg),
                "start= auto",
                &format!("DisplayName= {}", Self::DISPLAY_NAME),
            ])
            .output()
            .map_err(|e| format!("sc.exe calistirilamadi: {}", e))?;

        if output.status.success() {
            Ok("ApexConnect Windows Hizmeti basariyla kuruldu (Otomatik baslangic modunda).".into())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(format!("Hizmet kurulumu basarisiz: {} {}", stdout, stderr))
        }
    }

    /// Uninstall Windows service
    pub fn uninstall() -> Result<String, String> {
        let _ = Self::stop();

        let output = Command::new("sc.exe")
            .args(["delete", Self::SERVICE_NAME])
            .output()
            .map_err(|e| format!("sc.exe calistirilamadi: {}", e))?;

        if output.status.success() {
            Ok("ApexConnect Windows Hizmeti basariyla kaldirildi.".into())
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(format!("Hizmet kaldirilamadi: {}", stdout))
        }
    }

    /// Start Windows service
    pub fn start() -> Result<String, String> {
        let output = Command::new("sc.exe")
            .args(["start", Self::SERVICE_NAME])
            .output()
            .map_err(|e| format!("sc.exe calistirilamadi: {}", e))?;

        if output.status.success() {
            Ok("ApexConnect Windows Hizmeti baslatildi.".into())
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(format!("Hizmet baslatilamadi: {}", stdout))
        }
    }

    /// Stop Windows service
    pub fn stop() -> Result<String, String> {
        let output = Command::new("sc.exe")
            .args(["stop", Self::SERVICE_NAME])
            .output()
            .map_err(|e| format!("sc.exe calistirilamadi: {}", e))?;

        if output.status.success() {
            Ok("ApexConnect Windows Hizmeti durduruldu.".into())
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(format!("Hizmet durdurulamadi: {}", stdout))
        }
    }

    /// Query Windows service status
    pub fn status() -> String {
        match Command::new("sc.exe").args(["query", Self::SERVICE_NAME]).output() {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if stdout.contains("RUNNING") {
                    "RUNNING (Calisiyor - Kilit Ekrani ve UAC Yetkili)".into()
                } else if stdout.contains("STOPPED") {
                    "STOPPED (Durduruldu)".into()
                } else if stdout.contains("FAILED") || stdout.contains("1060") {
                    "NOT_INSTALLED (Kurulu Degil)".into()
                } else {
                    "UNKNOWN".into()
                }
            }
            Err(_) => "ERROR (sc.exe erisilemedi)".into(),
        }
    }
}
