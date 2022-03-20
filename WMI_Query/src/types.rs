use windows::{Win32::{System::Wmi::{IWbemServices_Vtbl, IWbemClassObject_Vtbl, IWbemServices, IUnsecuredApartment_Vtbl, IUnsecuredApartment, IWbemClassObject}}, core::{IUnknown, IUnknownVtbl}};

pub trait Vtable<T> where Self: windows::core::Interface<Vtable = T> {
    fn vtable(&self) -> &T {
        unsafe {
            windows::core::Interface::vtable(self)
        }
    }
}

impl Vtable<IWbemServices_Vtbl> for IWbemServices {}
impl Vtable<IUnsecuredApartment_Vtbl> for IUnsecuredApartment {}
impl Vtable<IUnknownVtbl> for IUnknown {}
impl Vtable<IWbemClassObject_Vtbl> for IWbemClassObject {}
