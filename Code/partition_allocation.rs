use std::io::{self, Write};
use std::process::{Command, ExitStatus};

fn main() {
    // Check for Administrator Privileges
    if !is_admin() {
        println!("[ERROR] This program must be run as an Administrator.");
        println!("Please right-click the executable and select 'Run as Administrator'.");
        prompt_input("\nPress Enter to exit...");
        return;
    }

    println!("--- OneClickFedoraInstaller ---");

    // 1. Select and Verify Disk
    let disk_num = loop {
        println!("\nAvailable disks:");
        run_powershell("Get-Disk | Format-Table Number, FriendlyName, Size");

        let input = prompt_input("Enter target disk number: ");
        if let Ok(num) = input.parse::<i32>() {
            let verify_script = format!(
                "$bp = Get-Partition | Where-Object IsBoot -eq $true; \
                 if(!$bp){{ exit 2 }}; \
                 if($bp.DiskNumber -eq {}){{ exit 0 }} else {{ exit 1 }}",
                num
            );

            if run_powershell(&verify_script).success() {
                println!("[SUCCESS] Disk verified");
                break num;
            }
        }
        println!("[ERROR] Invalid disk or Boot partition not found.");
    };

    // 2. Get Partition Size
    let size_gb = loop {
        let input = prompt_input("Enter Linux partition size in GB (min 6): ");
        match input.parse::<i32>() {
            Ok(s) if s >= 6 => break s,
            _ => println!("[ERROR] Minimum size is 6GB"),
        }
    };

    // 3. Shrink Windows Partition
    println!("\n[INFO] Shrinking Windows OS partition...");
    let shrink_script = format!(
        "$bp = Get-Partition | Where-Object IsBoot -eq $true; \
         $vol = Get-Volume -Partition $bp; \
         $sup = Get-PartitionSupportedSize -Partition $bp; \
         $new = $vol.Size - {}GB; \
         if($new -lt $sup.SizeMin){{ exit 1 }}; \
         Resize-Partition -Partition $bp -Size $new",
        size_gb
    );

    if !run_powershell(&shrink_script).success() {
        eprintln!("[ERROR] Shrink failed (not enough space)");
        return;
    }

    // 4. Create New Partition
    println!("[INFO] Creating Linux partition...");
    let create_script = format!(
        "$p = New-Partition -DiskNumber {} -Size {}GB -AssignDriveLetter; \
         Format-Volume -Partition $p -FileSystem NTFS -NewFileSystemLabel 'LINUXOS' -Confirm:$false",
        disk_num, size_gb
    );

    if run_powershell(&create_script).success() {
        println!("\n[FINISH] Linux partition created ({} GB)", size_gb);
    } else {
        println!("[ERROR] Partition creation failed");
    }
}

/// Checks if the current process has Administrator privileges
fn is_admin() -> bool {
    if cfg!(target_os = "windows") {
        // 'net session' returns 0 if admin, 1 if not.
        let status = Command::new("net")
            .arg("session")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        
        match status {
            Ok(s) => s.success(),
            Err(_) => false,
        }
    } else {
        false
    }
}

fn prompt_input(msg: &str) -> String {
    print!("{}", msg);
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    input.trim().to_string()
}

fn run_powershell(cmd: &str) -> ExitStatus {
    Command::new("powershell")
        .args(["-Command", cmd])
        .status()
        .expect("Failed to execute PowerShell")
}
