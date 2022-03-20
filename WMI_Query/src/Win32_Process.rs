use crate::ObjectWrapper::IWbemClassObjectWrapper;

/**
 * This struct represents the whole of the Win32_Process WMI object
 */

macro_rules! unwrap_enum {
    ($variant:expr, $enum_type:ident, $default:ty) => {{
        match ($variant.unwrap_or(&$crate::ObjectWrapper::ValueType::EMPTY)) {
            $crate::ObjectWrapper::ValueType::$enum_type(v) => v.clone(),
            $crate::ObjectWrapper::ValueType::EMPTY => <$default>::default(),
            _ => unreachable!()
        }
    }};
}

#[allow(non_camel_case_types)]
pub struct Win32_Process {
    pub Caption: String,
    pub CreationDate: String,
    pub CSCreationClassName: String,
    pub Description: String,
    pub CSName: String,
    pub VirtualSize: String,
    pub MaximumWorkingSetSize: i32,
    pub QuotaNonPagedPoolUsage: i32,
    pub ReadOperationCount: String,
    pub ExecutablePath: String,
    pub ParentProcessId: i32,
    pub ReadTransferCount: String,
    pub PeakWorkingSetSize: i32,
    pub UserModeTime: String,
    pub PageFileUsage: i32,
    pub OtherOperationCount: String,
    pub Name: String,
    pub HandleCount: i32,
    pub PrivatePageCount: String,
    pub QuotaPeakNonPagedPoolUsage: i32,
    pub MinimumWorkingSetSize: i32,
    pub PeakVirtualSize: String,
    pub QuotaPeakPagedPoolUsage: i32,
    pub SessionId: i32,
    pub WorkingSetSize: String,
    pub CreationClassName: String,
    pub OSCreationClassName: String,
    pub KernelModeTime: String,
    pub OtherTransferCount: String,
    pub Handle: String,
    pub PageFaults: i32,
    pub WriteOperationCount: String,
    pub WriteTransferCount: String,
    pub ProcessId: i32,
    pub ThreadCount: i32,
    pub OSName: String,
    pub Priority: i32,
    pub QuotaPagedPoolUsage: i32,
    pub PeakPageFileUsage: i32,
    pub WindowsVersion: String,
    pub CommandLine: String
}

impl From<IWbemClassObjectWrapper> for Win32_Process {
    fn from(obj: IWbemClassObjectWrapper) -> Self {
        let properties = obj.get_properties(true).unwrap().unwrap();

        Self {
            Caption:                    unwrap_enum!(properties.get("Caption"), BSTR, String),
            CreationDate:               unwrap_enum!(properties.get("CreationDate"), BSTR, String),
            CSCreationClassName:        unwrap_enum!(properties.get("CSCreationClassName"), BSTR, String),
            Description:                unwrap_enum!(properties.get("Description"), BSTR, String),
            CSName:                     unwrap_enum!(properties.get("CSName"), BSTR, String),
            VirtualSize:                unwrap_enum!(properties.get("VirtualSize"), BSTR, String),
            MaximumWorkingSetSize:      unwrap_enum!(properties.get("MaximumWorkingSetSize"), I4, i32),
            QuotaNonPagedPoolUsage:     unwrap_enum!(properties.get("QuotaNonPagedPoolUsage"), I4, i32),
            ReadOperationCount:         unwrap_enum!(properties.get("ReadOperationCount"), BSTR, String),
            ExecutablePath:             unwrap_enum!(properties.get("ExecutablePath"), BSTR, String),
            ParentProcessId:            unwrap_enum!(properties.get("ParentProcessId"), I4, i32),
            ReadTransferCount:          unwrap_enum!(properties.get("ReadTransferCount"), BSTR, String),
            PeakWorkingSetSize:         unwrap_enum!(properties.get("PeakWorkingSetSize"), I4, i32),
            UserModeTime:               unwrap_enum!(properties.get("UserModeTime"), BSTR, String),
            PageFileUsage:              unwrap_enum!(properties.get("PageFileUsage"), I4, i32),
            OtherOperationCount:        unwrap_enum!(properties.get("OtherOperationCount"), BSTR, String),
            Name:                       unwrap_enum!(properties.get("Name"), BSTR, String),
            HandleCount:                unwrap_enum!(properties.get("HandleCount"), I4, i32),
            PrivatePageCount:           unwrap_enum!(properties.get("PrivatePageCount"), BSTR, String),
            QuotaPeakNonPagedPoolUsage: unwrap_enum!(properties.get("QuotaPeakNonPagedPoolUsage"), I4, i32),
            MinimumWorkingSetSize:      unwrap_enum!(properties.get("MinimumWorkingSetSize"), I4, i32),
            PeakVirtualSize:            unwrap_enum!(properties.get("PeakVirtualSize"), BSTR, String),
            QuotaPeakPagedPoolUsage:    unwrap_enum!(properties.get("QuotaPeakPagedPoolUsage"), I4, i32),
            SessionId:                  unwrap_enum!(properties.get("SessionId"), I4, i32),
            WorkingSetSize:             unwrap_enum!(properties.get("WorkingSetSize"), BSTR, String),
            CreationClassName:          unwrap_enum!(properties.get("CreationClassName"), BSTR, String),
            OSCreationClassName:        unwrap_enum!(properties.get("OSCreationClassName"), BSTR, String),
            KernelModeTime:             unwrap_enum!(properties.get("KernelModeTime"), BSTR, String),
            OtherTransferCount:         unwrap_enum!(properties.get("OtherTransferCount"), BSTR, String),
            Handle:                     unwrap_enum!(properties.get("Handle"), BSTR, String),
            PageFaults:                 unwrap_enum!(properties.get("PageFaults"), I4, i32),
            WriteOperationCount:        unwrap_enum!(properties.get("WriteOperationCount"), BSTR, String),
            WriteTransferCount:         unwrap_enum!(properties.get("WriteTransferCount"), BSTR, String),
            ProcessId:                  unwrap_enum!(properties.get("ProcessId"), I4, i32),
            ThreadCount:                unwrap_enum!(properties.get("ThreadCount"), I4, i32),
            OSName:                     unwrap_enum!(properties.get("OSName"), BSTR, String),
            Priority:                   unwrap_enum!(properties.get("Priority"), I4, i32),
            QuotaPagedPoolUsage:        unwrap_enum!(properties.get("QuotaPagedPoolUsage"), I4, i32),
            PeakPageFileUsage:          unwrap_enum!(properties.get("PeakPageFileUsage"), I4, i32),
            WindowsVersion:             unwrap_enum!(properties.get("WindowsVersion"), BSTR, String),
            CommandLine:                unwrap_enum!(properties.get("CommandLine"), BSTR, String)
        }
    }
}
