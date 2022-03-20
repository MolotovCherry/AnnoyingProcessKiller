use crate::{utils::WMIError, ObjectWrapper::IWbemClassObjectWrapper};

use std::os::raw::c_long;
use async_channel::Sender;
use log::{warn, debug};
use windows::{core::{IUnknown, HRESULT, interface, implement}, Win32::{Foundation::{BSTR, E_POINTER}, System::Wmi::{IWbemClassObject, WBEM_S_NO_ERROR, WBEM_STATUS_COMPLETE}}};


// This is IWbemObjectSink
// must be declared as pub: https://github.com/microsoft/windows-rs/pull/1611
#[interface("7C857801-7381-11CF-884D-00AA004B2E24")]
pub unsafe trait IEventSink: IUnknown {
    unsafe fn Indicate(
        &self,
        lObjectCount: c_long,
        apObjArray: *mut *mut IWbemClassObject
    ) -> HRESULT;

    unsafe fn SetStatus(
        &self,
        lFlags: c_long,
        _hResult: HRESULT,
        _strParam: BSTR,
        _pObjParam: *mut IWbemClassObject
    ) -> HRESULT;
}

#[implement(IEventSink)]
pub struct EventSink {
    pub sender: Sender<Result<IWbemClassObjectWrapper, WMIError>>
}

impl EventSink {
    pub fn new(sender: Sender<Result<IWbemClassObjectWrapper, WMIError>>) -> Self {
        Self {
            sender
        }
    }
}

/// Implementation for [IWbemObjectSink](https://docs.microsoft.com/en-us/windows/win32/api/wbemcli/nn-wbemcli-iwbemobjectsink).
/// This [Sink](https://en.wikipedia.org/wiki/Sink_(computing))
/// receives asynchronously the result of the query, through Indicate calls.
/// When finished,the SetStatus method is called.
/// # <https://docs.microsoft.com/fr-fr/windows/win32/wmisdk/example--getting-wmi-data-from-the-local-computer-asynchronously>
impl IEventSink_Impl for EventSink {
    unsafe fn Indicate(
        &self,
        lObjectCount: c_long,
        apObjArray: *mut *mut IWbemClassObject
    ) -> HRESULT {
        debug!("entered indicate");
        debug!("Indicate call with {lObjectCount} objects");
        // Case of an incorrect or too restrictive query
        if lObjectCount <= 0 {
            return HRESULT(WBEM_S_NO_ERROR.0);
        }

        let lObjectCount = lObjectCount as usize;

        // The array memory of apObjArray is read-only
        // and is owned by the caller of the Indicate method.
        // IWbemClassWrapper::clone calls AddRef on each element
        // of apObjArray to make sure that they are not released,
        // according to COM rules.
        // https://docs.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemobjectsink-indicate
        // For error codes, see https://docs.microsoft.com/en-us/windows/win32/learnwin32/error-handling-in-com

        let slice = std::slice::from_raw_parts(
            apObjArray,
            lObjectCount
        );

        for obj in slice {
            let obj: &IWbemClassObject = core::mem::transmute(obj);

            let newobj = IWbemClassObjectWrapper::new(obj.Clone().unwrap());
            if let Err(e) = self.sender.try_send(Ok(newobj)) {
                warn!("Failed to send IWbemClassObject through channel: {:?}", e);
                return E_POINTER;
            }
        }

        HRESULT(WBEM_S_NO_ERROR.0)
    }

    unsafe fn SetStatus(
        &self,
        lFlags: c_long,
        _hResult: HRESULT,
        _strParam: BSTR,
        _pObjParam: *mut IWbemClassObject
    ) -> HRESULT {
        // SetStatus is called only once as flag=WBEM_FLAG_BIDIRECTIONAL in ExecQueryAsync
        // https://docs.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemobjectsink-setstatus
        // If you do not specify WBEM_FLAG_SEND_STATUS when calling your provider or service method,
        // you are guaranteed to receive one and only one call to SetStatus
        if lFlags == WBEM_STATUS_COMPLETE.0 {
            debug!("End of async result, closing transmitter");
            self.sender.close();
        }
        HRESULT(WBEM_S_NO_ERROR.0)
    }
}
