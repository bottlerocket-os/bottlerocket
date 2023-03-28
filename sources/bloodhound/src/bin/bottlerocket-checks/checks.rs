use bloodhound::results::{CheckStatus, Checker, CheckerMetadata, CheckerResult, Mode};
use bloodhound::*;

const PROC_MODULES_FILE: &str = "/proc/modules";
const PROC_CMDLINE_FILE: &str = "/proc/cmdline";
const SYSCTL_CMD: &str = "/usr/sbin/sysctl";
const MODPROBE_CMD: &str = "/bin/modprobe";

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR01010101Checker {}

impl Checker for BR01010101Checker {
    fn execute(&self) -> CheckerResult {
        let module_result = check_file_contains!(
            PROC_MODULES_FILE,
            &[" udf,"],
            "unable to parse modules to check for udf",
            "udf is currently loaded"
        );
        if module_result.status != CheckStatus::PASS {
            return module_result;
        }
        check_output_contains!(
            MODPROBE_CMD,
            ["-n", "-v", "udf"],
            &["install /bin/true"],
            "unable to parse modprobe output to check if udf is enabled",
            "modprobe for udf is not disabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure mounting of udf filesystems is disabled".to_string(),
            id: "1.1.1.1".to_string(),
            level: 2,
            name: "br01010101".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR01030100Checker {}

impl Checker for BR01030100Checker {
    fn execute(&self) -> CheckerResult {
        check_file_contains!(
            PROC_CMDLINE_FILE,
            &[
                "dm-mod.create=root,,,ro,0",
                "root=/dev/dm-0",
                "restart_on_corruption",
            ],
            "unable to verify cmdline includes dm-verity settings",
            "unable to verify dm-verity enforcement, settings not found"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure dm-verity is configured".to_string(),
            id: "1.3.1".to_string(),
            level: 1,
            name: "br01030100".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR01040100Checker {}

impl Checker for BR01040100Checker {
    fn execute(&self) -> CheckerResult {
        check_output_contains!(
            SYSCTL_CMD,
            ["fs.suid_dumpable"],
            &["fs.suid_dumpable = 0"],
            "unable to verify fs.suid_dumpable setting",
            "setuid core dumps are not disabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure setuid programs do not create core dumps".to_string(),
            id: "1.4.1".to_string(),
            level: 1,
            name: "br01040100".to_string(),
            mode: Mode::Automatic,
        }
    }
}
