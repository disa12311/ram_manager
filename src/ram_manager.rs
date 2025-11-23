use std::collections::HashMap;
use windows::Win32::Foundation::{HANDLE, CloseHandle};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_ALL_ACCESS, PROCESS_SET_QUOTA, PROCESS_QUERY_INFORMATION,
    SetPriorityClass, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS, HIGH_PRIORITY_CLASS,
};
use windows::Win32::System::Memory::{SetProcessWorkingSetSize, EmptyWorkingSet};
use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use sysinfo::{System, ProcessExt, SystemExt, Pid};

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory_mb: f64,
    pub working_set_mb: f64,
    pub status: ProcessStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProcessStatus {
    Normal,
    Pinned,
    Trimmed,
    Limited,
}

impl ProcessStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ProcessStatus::Normal => "Bình thường",
            ProcessStatus::Pinned => "Đã ghim",
            ProcessStatus::Trimmed => "Đã trim",
            ProcessStatus::Limited => "Giới hạn",
        }
    }

    pub fn color(&self) -> [u8; 3] {
        match self {
            ProcessStatus::Normal => [128, 128, 128],
            ProcessStatus::Pinned => [46, 204, 113],
            ProcessStatus::Trimmed => [52, 152, 219],
            ProcessStatus::Limited => [230, 126, 34],
        }
    }
}

pub struct RamManager {
    system: System,
    process_states: HashMap<u32, ProcessStatus>,
}

impl RamManager {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        RamManager {
            system: sys,
            process_states: HashMap::new(),
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    pub fn get_system_info(&self) -> SystemInfo {
        SystemInfo {
            total_ram_gb: self.system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
            used_ram_gb: self.system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
            available_ram_gb: self.system.available_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
        }
    }

    pub fn list_processes(&mut self) -> Vec<ProcessInfo> {
        self.refresh();
        let mut processes: Vec<ProcessInfo> = self
            .system
            .processes()
            .iter()
            .map(|(pid, proc)| {
                let pid_u32 = pid.as_u32();
                ProcessInfo {
                    pid: pid_u32,
                    name: proc.name().to_string(),
                    memory_mb: proc.memory() as f64 / 1024.0 / 1024.0,
                    working_set_mb: proc.memory() as f64 / 1024.0 / 1024.0,
                    status: self
                        .process_states
                        .get(&pid_u32)
                        .cloned()
                        .unwrap_or(ProcessStatus::Normal),
                }
            })
            .collect();

        processes.sort_by(|a, b| b.memory_mb.partial_cmp(&a.memory_mb).unwrap());
        processes
    }

    pub fn pin_to_ram(&mut self, pid: u32, working_set_mb: usize) -> Result<String, String> {
        unsafe {
            let handle = OpenProcess(PROCESS_SET_QUOTA | PROCESS_QUERY_INFORMATION, false, pid)
                .map_err(|e| format!("Không thể mở tiến trình: {}", e))?;

            let min_size = (working_set_mb * 1024 * 1024) as usize;
            let max_size = (working_set_mb * 2 * 1024 * 1024) as usize;

            SetProcessWorkingSetSize(handle, min_size, max_size)
                .map_err(|e| format!("Không thể đặt working set: {}", e))?;

            SetPriorityClass(handle, HIGH_PRIORITY_CLASS)
                .map_err(|e| format!("Không thể đặt priority: {}", e))?;

            let _ = CloseHandle(handle);

            self.process_states.insert(pid, ProcessStatus::Pinned);
            Ok(format!(
                "✅ Đã ghim PID {} vào RAM ({} MB)",
                pid, working_set_mb
            ))
        }
    }

    pub fn trim_working_set(&mut self, pid: u32) -> Result<String, String> {
        unsafe {
            let handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid)
                .map_err(|e| format!("Không thể mở tiến trình: {}", e))?;

            let before = self.get_process_memory_info_internal(handle)?;
            let before_ws = before.WorkingSetSize as f64 / 1024.0 / 1024.0;

            EmptyWorkingSet(handle).map_err(|e| format!("Không thể trim working set: {}", e))?;

            SetPriorityClass(handle, IDLE_PRIORITY_CLASS)
                .map_err(|e| format!("Không thể đặt priority: {}", e))?;

            std::thread::sleep(std::time::Duration::from_millis(300));

            let after = self.get_process_memory_info_internal(handle)?;
            let after_ws = after.WorkingSetSize as f64 / 1024.0 / 1024.0;
            let freed = (before_ws - after_ws).max(0.0);

            let _ = CloseHandle(handle);

            self.process_states.insert(pid, ProcessStatus::Trimmed);
            Ok(format!(
                "✅ Đã trim PID {}\n📉 Trước: {:.1} MB → Sau: {:.1} MB\n💾 Giải phóng: {:.1} MB",
                pid, before_ws, after_ws, freed
            ))
        }
    }

    pub fn limit_resources(&mut self, pid: u32, max_ws_mb: usize) -> Result<String, String> {
        unsafe {
            let handle = OpenProcess(PROCESS_SET_QUOTA | PROCESS_QUERY_INFORMATION, false, pid)
                .map_err(|e| format!("Không thể mở tiến trình: {}", e))?;

            let max_size = (max_ws_mb * 1024 * 1024) as usize;
            let min_size = (max_ws_mb / 2 * 1024 * 1024) as usize;

            SetProcessWorkingSetSize(handle, min_size, max_size)
                .map_err(|e| format!("Không thể giới hạn working set: {}", e))?;

            SetPriorityClass(handle, IDLE_PRIORITY_CLASS)
                .map_err(|e| format!("Không thể đặt priority: {}", e))?;

            let _ = CloseHandle(handle);

            self.process_states.insert(pid, ProcessStatus::Limited);
            Ok(format!(
                "✅ Đã giới hạn PID {} (Max: {} MB, Priority: IDLE)",
                pid, max_ws_mb
            ))
        }
    }

    pub fn restore_process(&mut self, pid: u32) -> Result<String, String> {
        unsafe {
            let handle = OpenProcess(PROCESS_SET_QUOTA | PROCESS_QUERY_INFORMATION, false, pid)
                .map_err(|e| format!("Không thể mở tiến trình: {}", e))?;

            SetProcessWorkingSetSize(handle, usize::MAX, usize::MAX)
                .map_err(|e| format!("Không thể reset working set: {}", e))?;

            SetPriorityClass(handle, NORMAL_PRIORITY_CLASS)
                .map_err(|e| format!("Không thể đặt priority: {}", e))?;

            let _ = CloseHandle(handle);

            self.process_states.remove(&pid);
            Ok(format!(
                "✅ Đã khôi phục PID {} về trạng thái bình thường",
                pid
            ))
        }
    }

    fn get_process_memory_info_internal(
        &self,
        handle: HANDLE,
    ) -> Result<PROCESS_MEMORY_COUNTERS, String> {
        unsafe {
            let mut pmc = PROCESS_MEMORY_COUNTERS::default();
            GetProcessMemoryInfo(
                handle,
                &mut pmc,
                std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            )
            .map_err(|e| format!("Không thể lấy thông tin bộ nhớ: {}", e))?;
            Ok(pmc)
        }
    }
}

pub struct SystemInfo {
    pub total_ram_gb: f64,
    pub used_ram_gb: f64,
    pub available_ram_gb: f64,
}