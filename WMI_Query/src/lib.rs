#![allow(non_snake_case)]

mod event_sink;
mod utils;
mod types;
mod ObjectWrapper;
pub mod Win32_Process;
pub use Win32_Process::*;

use ObjectWrapper::IWbemClassObjectWrapper;
use event_sink::EventSink;
use log::debug;
use utils::WMIError;

use std::{error::Error, ops::Deref};
use async_channel::{unbounded, Receiver};

use windows::{
    Win32::{
        System::{
            Wmi::{
                WbemLocator, IWbemLocator, IUnsecuredApartment, IWbemServices, UnsecuredApartment,
                IWbemObjectSink,
                WBEM_FLAG_SEND_STATUS, WBEM_E_UNPARSABLE_QUERY
            },
            Com::{
                CoInitializeEx, COINIT_MULTITHREADED, RPC_C_AUTHN_LEVEL_DEFAULT, RPC_C_IMP_LEVEL_IMPERSONATE, EOAC_NONE,
                CoCreateInstance, CLSCTX_INPROC_SERVER,
                CoSetProxyBlanket, RPC_C_AUTHN_LEVEL_CALL, CLSCTX_LOCAL_SERVER
            },
            Rpc::{
                RPC_C_AUTHN_WINNT, RPC_C_AUTHN_NONE
            }
        },
        Foundation::{
            BSTR
        }
    }, core::{Interface, IUnknown, IntoParam}
};


pub struct WMIConnection {
    // service used for actual calls
    pub pSvc: IWbemServices,
    pUnsecApp: IUnsecuredApartment
}

impl WMIConnection {
    pub fn new() -> Result<Self, windows::core::Error> {
        unsafe {
            CoInitializeEx(std::ptr::null(), COINIT_MULTITHREADED)?;

            //
            // https://github.com/microsoft/win32metadata/issues/837
            // https://github.com/microsoft/windows-rs/issues/1610
            //
            #[link(name = "windows")]
            extern "system" {
                fn CoInitializeSecurity(
                    psecdesc: *const windows::Win32::Security::SECURITY_DESCRIPTOR,
                    cauthsvc: i32,
                    asauthsvc: *const windows::Win32::System::Com::SOLE_AUTHENTICATION_SERVICE,
                    preserved1: *const ::core::ffi::c_void,
                    dwauthnlevel: windows::Win32::System::Com::RPC_C_AUTHN_LEVEL,
                    dwimplevel: windows::Win32::System::Com::RPC_C_IMP_LEVEL,
                    pauthlist: *const ::core::ffi::c_void,
                    dwcapabilities: windows::Win32::System::Com::EOLE_AUTHENTICATION_CAPABILITIES,
                    preserved3: *const ::core::ffi::c_void
                ) -> ::windows::core::HRESULT;
            }

            //
            // https://github.com/microsoft/win32metadata/issues/837
            // https://github.com/microsoft/windows-rs/issues/1610
            //
            CoInitializeSecurity(
                std::ptr::null(),
                -1,
                std::ptr::null(),
                std::ptr::null(),
                RPC_C_AUTHN_LEVEL_DEFAULT,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                std::ptr::null(),
                EOAC_NONE,
                std::ptr::null()
            ).ok()?;

            // can't put in -1 due to bug on 0.34.0
            //
            // https://github.com/microsoft/win32metadata/issues/837
            // https://github.com/microsoft/windows-rs/issues/1610
            //
            /*
            CoInitializeSecurity(
                std::ptr::null(),
                // should be -1
                &[],
                std::ptr::null(),
                RPC_C_AUTHN_LEVEL_DEFAULT,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                std::ptr::null(),
                EOAC_NONE,
                std::ptr::null()
            )?;
            */

            let pLoc: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)?;

            let pSvc = pLoc.ConnectServer(
                BSTR::from("ROOT\\CIMV2"),
                None,
                None,
                None,
                0,
                None,
                None
            )?;

            CoSetProxyBlanket(
                &pSvc,
                RPC_C_AUTHN_WINNT,
                RPC_C_AUTHN_NONE,
                None,
                RPC_C_AUTHN_LEVEL_CALL,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                std::ptr::null(),
                EOAC_NONE
            )?;

            let pUnsecApp: IUnsecuredApartment = CoCreateInstance(&UnsecuredApartment, None, CLSCTX_LOCAL_SERVER)?;

            Ok(Self {
                pSvc,
                pUnsecApp
            })
        }
    }

    pub fn exec_notification_query_async(&self, query: &str) -> Result<AsyncQueryReceiver, Box<dyn Error>> {
        let (tx, rx) = unbounded();

        let event_sink = EventSink::new(tx);
        let unknown: IUnknown = event_sink.into();

        let sink: IWbemObjectSink;
        unsafe {
            let pStubUnk: IUnknown = self.pUnsecApp.CreateObjectStub(unknown)?;

            sink = pStubUnk.cast()?;

            //let pctx: IWbemContext = CoCreateInstance(&WbemContext, None, CLSCTX_LOCAL_SERVER)?;

            let res = (Interface::vtable(&self.pSvc).ExecNotificationQueryAsync)(
                core::mem::transmute_copy(&self.pSvc),
                BSTR::from("WQL").into_param().abi(),
                BSTR::from(query).into_param().abi(),
                WBEM_FLAG_SEND_STATUS.0,
                std::ptr::null_mut(),
                core::mem::transmute_copy(&sink)
            ).ok();

            if let Err(e) = res {
                if e.code().0 == WBEM_E_UNPARSABLE_QUERY.0 {
                    return Err(Box::new(WMIError::WbemUnparsableQuery))
                }
            }
        }

        let asyncreceiver = AsyncQueryReceiver {
            receiver: rx,
            pSvc: &self.pSvc,
            pStubSink: sink
        };

        Ok(asyncreceiver)
    }
}

pub struct AsyncQueryReceiver<'a> {
    pub receiver: Receiver<Result<IWbemClassObjectWrapper, WMIError>>,
    pub pSvc: &'a IWbemServices,
    pub pStubSink: IWbemObjectSink
}

impl Deref for AsyncQueryReceiver<'_> {
    type Target = Receiver<Result<IWbemClassObjectWrapper, WMIError>>;

    fn deref(&self) -> &Self::Target {
        &self.receiver
    }
}

impl<'a> Drop for AsyncQueryReceiver<'_> {
    fn drop(&mut self) {
        debug!("Canceling async call");
        unsafe {
            let _ = self.pSvc.CancelAsyncCall(&self.pStubSink);
        }
    }
}
