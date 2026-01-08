#include <stdio.h>
#include <stdlib.h>

int main() {

    char command[2048];
    int diskNum;
    int sizeGB;

    printf("============= OneClickFedoraInstaller =============\n");

    while (1) {
        printf("\nAvailable disks:\n");
        system("powershell -Command \"Get-Disk | Format-Table Number, FriendlyName, Size\"");

        printf("\nEnter target disk number (must contain Windows OS): ");
        scanf("%d", &diskNum);

        snprintf(command, sizeof(command),
            "powershell -Command \""
            "$bp = Get-Partition | Where-Object IsBoot -eq $true;"
            "if(!$bp){ exit 2 };"
            "if($bp.DiskNumber -eq %d){ exit 0 } else { exit 1 }\"",
            diskNum
        );

        if (system(command) == 0) {
            printf("✅ Disk verified\n");
            break;
        }

        printf("❌ Wrong disk. Please try again.\n");
    }

    while (1) {
        printf("\nEnter Linux partition size in GB (minimum 6): ");
        scanf("%d", &sizeGB);

        if (sizeGB >= 6) break;

        printf("❌ Minimum size is 6GB\n");
    }

    // SHRINK WINDOWS OS PARTITION
    printf("\n[INFO] Shrinking Windows OS partition...\n");

    snprintf(command, sizeof(command),
        "powershell -Command \""
        "$bp = Get-Partition | Where-Object IsBoot -eq $true;"
        "$vol = Get-Volume -Partition $bp;"
        "$sup = Get-PartitionSupportedSize -Partition $bp;"
        "$new = $vol.Size - %dGB;"
        "if($new -lt $sup.SizeMin){ exit 1 };"
        "Resize-Partition -Partition $bp -Size $new\"",
        sizeGB
    );

    if (system(command) != 0) {
        printf("❌ Shrink failed (not enough space)\n");
        return 1;
    }

    printf("✔ Windows partition shrunk\n");

    //CREATE LINUX PARTITION
    printf("[INFO] Creating Linux partition...\n");

    snprintf(command, sizeof(command),
        "powershell -Command \""
        "$p = New-Partition -DiskNumber %d -Size %dGB -AssignDriveLetter;"
        "Format-Volume -Partition $p "
        "-FileSystem NTFS "
        "-NewFileSystemLabel 'LINUXOS' "
        "-Confirm:$false\"",
        diskNum, sizeGB
    );

    if (system(command) != 0) {
        printf("❌ Linux partition creation failed\n");
        return 1;
    }

    printf("\n✅ SUCCESS: Linux partition created (%d GB)\n", sizeGB);

    return 0;
}
