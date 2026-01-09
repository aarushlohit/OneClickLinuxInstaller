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
            // Verify if the chosen disk contains the Windows/Boot partition
            let verify_script = format!(
                "$p = Get-Partition -DiskNumber {}; \
                 if ($p | Where-Object {{ $_.DriveLetter -eq 'C' -or $_.IsBoot -eq $true }}) {{ exit 0 }} else {{ exit 1 }}",
                num
            );

            if run_powershell(&verify_script).success() {
                println!("[SUCCESS] Disk verified");
                break num;
            }
        }
        println!("[ERROR] Invalid disk or Boot/C: partition not found on this disk.");
    };

    // 2. Get Partition Size
    let size_gb = loop {
        let input = prompt_input("Enter Linux partition size in GB (min 6): ");
        match input.parse::<i64>() {
            Ok(s) if s >= 6 => break s,
            _ => println!("[ERROR] Minimum size is 6GB"),
        }
    };

    // 3. Shrink Windows Partition
    println!("\n[INFO] Shrinking Windows OS partition...");
    let shrink_script = format!(
        "$bp = Get-Partition | Where-Object {{ $_.DriveLetter -eq 'C' }}; \
         if (-not $bp) {{ $bp = Get-Partition | Where-Object {{ $_.IsBoot -eq $true }} }}; \
         if (-not $bp) {{ exit 1 }}; \
         $vol = Get-Volume -Partition $bp; \
         $sup = Get-PartitionSupportedSize -Partition $bp; \
         $shrinkSize = [long]{} * 1GB; \
         $newSize = $vol.Size - $shrinkSize; \
         if($newSize -lt $sup.SizeMin){{ exit 2 }}; \
         Resize-Partition -Partition $bp -Size $newSize",
        size_gb
    );

    let shrink_status = run_powershell(&shrink_script);
    if !shrink_status.success() {
        match shrink_status.code() {
            Some(1) => eprintln!("[ERROR] Could not find the Windows partition (C:)."),
            Some(2) => eprintln!("[ERROR] Not enough shrinkable space. Windows has unmovable files at the end of the disk."),
            _ => eprintln!("[ERROR] Shrink failed due to an unknown PowerShell error."),
        }
        prompt_input("\nPress Enter to exit...");
        return;
    }

    // 4. Create New Partition
    println!("[INFO] Creating Linux partition...");
    // We use -UseMaximumSize to fill the unallocated space we just created
    let create_script = format!(
        "$p = New-Partition -DiskNumber {} -UseMaximumSize -AssignDriveLetter; \
         Format-Volume -Partition $p -FileSystem NTFS -NewFileSystemLabel 'LINUXOS' -Confirm:$false",
        disk_num
    );

    if run_powershell(&create_script).success() {
        println!("\n[FINISH] Linux partition created successfully!");
    } else {
        println!("[ERROR] Partition creation failed. The space was shrunk, but the new partition couldn't be initialized.");
    }

    prompt_input("\nPress Enter to finish...");
}

fn is_admin() -> bool {
    if cfg!(target_os = "windows") {
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
        .args(["-NoProfile", "-Command", cmd])
        .status()
        .expect("Failed to execute PowerShell")
}