use std::{error::Error, ffi::c_void, collections::{HashMap, VecDeque}};

use windows::{
    Win32::{
        System::{
            Wmi::{
                IWbemClassObject, WBEM_FLAG_ALWAYS, CIM_OBJECT
            },
            Com::{VARIANT, SAFEARRAY},
            Ole::{
                SafeArrayAccessData, SafeArrayUnaccessData, VT_UNKNOWN, VARENUM,
                VT_BSTR, VT_I8, VT_EMPTY, VT_I1, VT_I2, VT_I4, VT_BOOL, VT_SAFEARRAY, VT_NULL,
                VT_UI1, VT_UI2, VT_UI4, VT_UI8, VT_INT, VT_UINT, VT_VOID, VT_R4, VT_R8
            }
        },
        Foundation::BSTR
    },
    core::{
        PCWSTR, Interface
    }
};

use crate::utils::WMIError;


#[allow(non_camel_case_types)]
#[non_exhaustive]
#[derive(Debug)]
pub enum ValueType {
    EMPTY,
    CIM_OBJECT(IWbemClassObjectWrapper),
    BSTR(String),
    I1(i8),
    I2(i16),
    I4(i32),
    I8(i64),
    UI1(u8),
    UI2(u16),
    UI4(u32),
    UI8(u64),
    R4(f32),
    R8(f64),
    BOOL(bool),
    SAFEARRAY(SAFEARRAY),
    //CIM_STRING_ARRAY(Vec<String>)
}


#[derive(Debug)]
pub struct IWbemClassObjectWrapper {
    obj: IWbemClassObject
}

impl IWbemClassObjectWrapper {
    pub fn new(obj: IWbemClassObject) -> Self {
        Self {
            obj
        }
    }

    pub fn get_property_names(&self) -> Result<Option<Vec<String>>, Box<dyn Error>> {
        let mut arrs = VecDeque::new();

        unsafe {
            let _safearray = self.obj.GetNames(
                PCWSTR::default(),
                WBEM_FLAG_ALWAYS.0,// | WBEM_FLAG_NONSYSTEM_ONLY.0,
                &VARIANT::default() as *const _ as *const _
            )?;

            let mut ptr: *mut c_void = std::mem::zeroed();

            SafeArrayAccessData(
                _safearray as *const _,
                &mut ptr as *mut _
            )?;

            let safearray = *_safearray;

            for i in 0..safearray.cDims as usize {
                let slice = std::slice::from_raw_parts(
                    ptr as *mut BSTR,
                    safearray.rgsabound[i].cElements as usize
                );
                arrs.push_back(slice.into_iter().map(|f| f.to_string()).collect::<Vec<String>>());
            }

            SafeArrayUnaccessData(
                _safearray
            )?;
        }

        if arrs.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(arrs.pop_front().unwrap()))
        }
    }

    pub fn get_property(&self, name: &str) -> Result<Option<(String, ValueType)>, Box<dyn Error>> {
        let mut variant = VARIANT::default();
        let property = BSTR::from(name);
        let property = property.as_wide();
        let mut var_type = 0i32;

        let value = unsafe {
            self.obj.Get(
                PCWSTR(property.as_ptr()),
                0,
                &mut variant as *mut _,
                &mut var_type as *mut _,
                std::ptr::null_mut()
            )?;

            match VARENUM(variant.Anonymous.Anonymous.vt as i32) {
                VT_UNKNOWN => {
                    // this unknown type is generally an embedded object
                    if var_type != CIM_OBJECT.0 {
                        return Err(Box::new(WMIError::NotCimObject))
                    }

                    // convert embedded object to IUnknown, then cast to IWbemClassObject
                    let pVal = variant.Anonymous.Anonymous.Anonymous.punkVal.as_ref().unwrap();
                    let embeddedObject = pVal.cast::<IWbemClassObject>()?;
                    ValueType::CIM_OBJECT(Self::new(embeddedObject))
                }

                VT_BSTR => {
                    let bstring = &*variant.Anonymous.Anonymous.Anonymous.bstrVal;
                    let string = bstring.to_string();
                    ValueType::BSTR(string)
                }

                // float 32
                VT_R4 => {
                    ValueType::R4(variant.Anonymous.Anonymous.Anonymous.fltVal)
                }

                // double 64
                VT_R8 => {
                    ValueType::R8(variant.Anonymous.Anonymous.Anonymous.dblVal)
                }

                // 1 byte signed
                VT_I1 => {
                    todo!("VT_I1 does not exist?")
                }

                // 2 byte signed
                VT_I2 => {
                    ValueType::I2(variant.Anonymous.Anonymous.Anonymous.iVal)
                }

                // 4 byte signed
                VT_I4 | VT_INT => {
                    ValueType::I4(variant.Anonymous.Anonymous.Anonymous.intVal)
                }

                // 8 byte signed
                VT_I8 => {
                    ValueType::I8(variant.Anonymous.Anonymous.Anonymous.llVal)
                }

                // 1 byte unsigned
                VT_UI1 => {
                    ValueType::UI1(variant.Anonymous.Anonymous.Anonymous.bVal)
                }

                // 2 bytes unsigned
                VT_UI2 => {
                    ValueType::UI2(variant.Anonymous.Anonymous.Anonymous.uiVal)
                }

                // 4 bytes unsigned
                VT_UI4 | VT_UINT => {
                    ValueType::UI4(variant.Anonymous.Anonymous.Anonymous.uintVal)
                }

                // 8 bytes unsigned
                VT_UI8 => {
                    ValueType::UI8(variant.Anonymous.Anonymous.Anonymous.ullVal)
                }

                VT_BOOL => {
                    ValueType::BOOL(variant.Anonymous.Anonymous.Anonymous.boolVal != 0)
                }

                VT_SAFEARRAY => {
                    let _safearray = variant.Anonymous.Anonymous.Anonymous.parray;
                    let safearray = *_safearray;

                    ValueType::SAFEARRAY(safearray)
                }

                // Nothing
                VT_EMPTY => {
                    return Ok(None)
                }

                // NULL
                VT_NULL => {
                    return Ok(None)
                }

                VT_VOID => {
                    todo!("VT_VOID")
                }

                /*CIM_FLAG_ARRAY | CIM_STRING => {
                    let mut arrs = VecDeque::new();
                    let _safearray = variant.Anonymous.Anonymous.Anonymous.parray;
                    let safearray = *_safearray;

                    let mut ptr: *mut c_void = std::mem::zeroed();

                    SafeArrayAccessData(
                        _safearray as *const _,
                        &mut ptr as *mut _
                    )?;

                    for i in 0..safearray.cDims as usize {
                        let slice = std::slice::from_raw_parts(
                            ptr as *mut BSTR,
                            safearray.rgsabound[i].cElements as usize
                        );
                        arrs.push_back(slice.into_iter().map(|f| f.to_string()).collect::<Vec<String>>());
                    }

                    SafeArrayUnaccessData(
                        _safearray
                    )?;


                }*/

                v => {
                    //todo!("TODO: Add another ValueType!: {v:?}");
                    println!("Warning: Skipped {name} with type 0x{:x?}", v.0);
                    return Ok(None)
                },
            }
        };

        Ok(
            Some(
                (
                    name.to_string(),
                    value
                )
            )
        )
    }

    pub fn get_embedded_object(&self, name: &str) -> Result<IWbemClassObjectWrapper, Box<dyn Error>> {
        let mut variant = VARIANT::default();
        let property = BSTR::from(name);
        let property = property.as_wide();
        let mut cim_type = 0i32;

        let processObject = unsafe {
            self.obj.Get(
                PCWSTR(property.as_ptr()),
                0,
                &mut variant as *mut _,
                &mut cim_type as *mut _,
                std::ptr::null_mut()
            )?;

            if cim_type != CIM_OBJECT.0 {
                return Err(Box::new(WMIError::NotCimObject))
            }

            // convert embedded object to IUnknown, then cast to IWbemClassObject
            let pVal = variant.Anonymous.Anonymous.Anonymous.punkVal.as_ref().unwrap();
            let processObject = pVal.cast::<IWbemClassObject>()?;

            Self::new(processObject)
        };

        Ok(processObject)
    }

    pub fn get_properties(&self, skip_system: bool) -> Result<Option<HashMap<String, ValueType>>, Box<dyn Error>> {
        let properties = self.get_property_names()?.unwrap_or_default();

        if properties.len() == 0 {
            return Ok(None)
        }

        let mut hashmap = HashMap::<String, ValueType>::new();
        for property in properties {
            if skip_system && property.starts_with("__"){
                continue
            }

            if let Some(p) = self.get_property(&property)? {
                hashmap.insert(p.0, p.1);
            }
        }

        Ok(Some(hashmap))
    }
}
