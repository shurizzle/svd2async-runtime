use crate::utils::*;

pub fn run(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let interrupts = svd_parser::parse(&std::fs::read_to_string(file)?)?
        .peripherals
        .iter()
        .flat_map(|p| {
            p.interrupt
                .iter()
                .filter(|i| !starts_with_case_insensitive(i.name.as_str(), "reserved"))
                .cloned()
                .map(|mut i| {
                    i.name = i.name.to_lowercase();
                    i.description = i.description.map(|mut d| {
                        d.in_place(str::trim);
                        d
                    });
                    i
                })
        })
        .collect::<Vec<_>>();

    println!("use avr_device::interrupt::CriticalSection;");
    println!();

    println!("pub trait Runtime: crate::runtime::Ready {{");
    println!("    type Result: crate::tuple::Tuple;");

    println!();
    println!("    fn init(&mut self, cs: &CriticalSection) -> Self::Result;");
    println!();
    println!("    fn snapshot(&mut self, cs: &CriticalSection);");
    println!();
    println!("    fn idle(&self);");
    println!();
    println!("    fn wake(&mut self);");
    println!();
    println!("    fn shutdown(&self);");
    for i in &interrupts {
        println!();
        if let Some(desc) = i.description.as_ref() {
            for l in desc.lines() {
                println!("    /// {}", l);
            }
            println!("    ///");
        }
        println!("    /// # Safety");
        println!("    /// Interrupts are marked unsafe and executed in critical section");
        println!("    #[inline(always)]");
        println!(
            "    unsafe fn {}(&mut self, _cs: &CriticalSection) {{}}",
            i.name
        );
    }
    println!("}}");

    println!();
    println!("mod interrupts {{");
    println!("    use avr_device::interrupt::CriticalSection;");
    for int in &interrupts {
        println!("    #[doc(hidden)]");
        println!("    #[export_name = \"__vector_{}\"]", int.value);
        println!(
            "    unsafe extern \"avr-interrupt\" fn __vector_{}() {{",
            int.value
        );
        println!(
            "        crate::executor::__private::RUNTIME.{}(&CriticalSection::new())",
            int.name
        );
        println!("    }}");
    }
    println!("}}");

    let vtable_trampoline = |name: &str| {
        println!();
        println!("    #[inline(always)]");
        println!(
            "    pub unsafe fn {}<R: super::Runtime>(ptr: *mut ()) {{",
            name
        );
        println!("        (*(ptr as *mut R)).{}()", name);
        println!("    }}");
    };
    let vtable_cs_trampoline = |name: &str| {
        println!();
        println!("    #[inline(always)]");
        println!(
            "    pub unsafe fn {}<R: super::Runtime>(ptr: *mut (), cs: &CriticalSection) {{",
            name
        );
        println!("        (*(ptr as *mut R)).{}(cs)", name);
        println!("    }}");
    };

    println!();
    println!("mod vtable {{");
    println!("    use avr_device::interrupt::CriticalSection;");
    vtable_cs_trampoline("snapshot");
    vtable_trampoline("idle");
    vtable_trampoline("wake");
    vtable_trampoline("shutdown");
    println!();
    println!("    #[inline(always)]");
    println!("    pub unsafe fn is_ready<R: super::Runtime>(ptr: *mut (), cs: &CriticalSection) -> bool {{");
    println!("        (*(ptr as *mut R)).is_ready(cs)");
    println!("    }}");
    for i in &interrupts {
        vtable_cs_trampoline(&i.name);
    }
    println!("}}");

    println!();
    println!("#[repr(C)]");
    println!("struct Vtable {{");
    println!("    pub snapshot: unsafe fn(*mut (), &CriticalSection),");
    println!("    pub idle: unsafe fn(*mut ()),");
    println!("    pub wake: unsafe fn(*mut ()),");
    println!("    pub shutdown: unsafe fn(*mut ()),");
    println!("    pub is_ready: unsafe fn(*mut (), &CriticalSection) -> bool,");
    for i in &interrupts {
        println!("    pub {}: unsafe fn(*mut (), &CriticalSection),", i.name);
    }
    println!("}}");

    let vtable_entry = |name: &str| {
        println!("        {}: vtable::{}::<R>,", name, name);
    };

    println!();
    println!("#[inline(always)]");
    println!("const fn vtable<R: Runtime>() -> &'static Vtable {{");
    println!("    &Vtable {{");
    vtable_entry("snapshot");
    vtable_entry("idle");
    vtable_entry("wake");
    vtable_entry("shutdown");
    vtable_entry("is_ready");
    for i in &interrupts {
        vtable_entry(&i.name);
    }
    println!("    }}");
    println!("}}");

    let call_trampoline = |name: &str| {
        println!("    #[allow(dead_code)]");
        println!("    #[inline(always)]");
        println!("    pub fn {}(&self) {{", name);
        println!(
            "        unsafe {{ ((*(self.vtable)).{})(self.data) }}",
            name
        );
        println!("    }}");
    };
    let call_cs_trampoline = |name: &str, allow_dead_code: bool| {
        if allow_dead_code {
            println!("    #[allow(dead_code)]");
        }
        println!("    #[inline(always)]");
        println!("    pub fn {}(&self, cs: &CriticalSection) {{", name);
        println!(
            "        unsafe {{ ((*(self.vtable)).{})(self.data, cs) }}",
            name
        );
        println!("    }}");
    };

    println!();
    println!("#[repr(C)]");
    println!("pub struct RawRuntime {{");
    println!("    data: *mut (),");
    println!("    vtable: *const Vtable,");
    println!("}}");
    println!();
    println!("unsafe impl Sync for RawRuntime {{}}");
    println!();
    println!("impl RawRuntime {{");
    println!("    #[inline(always)]");
    println!("    pub const fn uninit() -> Self {{");
    println!("        Self {{");
    println!("            data: 0 as *mut (),");
    println!("            vtable: 0 as *const Vtable,");
    println!("        }}");
    println!("    }}");
    println!();
    println!("    #[inline(always)]");
    println!("    pub const fn new<R: Runtime>(runtime: &R) -> Self {{");
    println!("        Self {{");
    println!("            data: runtime as *const R as *const () as *mut (),");
    println!("            vtable: vtable::<R>(),");
    println!("        }}");
    println!("    }}");
    println!();
    println!();
    call_cs_trampoline("snapshot", true);
    println!();
    call_trampoline("idle");
    println!();
    call_trampoline("wake");
    println!();
    call_trampoline("shutdown");
    println!();
    println!("    #[allow(dead_code)]");
    println!("    #[inline(always)]");
    println!("    pub fn is_ready(&self, cs: &CriticalSection) -> bool {{",);
    println!("        unsafe {{ ((*(self.vtable)).is_ready)(self.data, cs) }}",);
    println!("    }}");
    for i in &interrupts {
        println!();
        if let Some(desc) = i.description.as_ref() {
            for l in desc.lines() {
                println!("    /// {}", l);
            }
        }
        call_cs_trampoline(&i.name, false);
    }
    println!();
    println!("    #[doc(hidden)]");
    println!("    #[inline(always)]");
    println!("    pub unsafe fn from_ptr<'a>(p: *const ()) -> &'a Self {{");
    println!("        &*(p as *const Self)");
    println!("    }}");
    println!("}}");

    Ok(())
}
