use crate::utils::*;

pub fn run(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let interrupts = svd_parser::parse(&std::fs::read_to_string(file)?)?
        .peripherals
        .iter()
        .flat_map(|p| {
            p.interrupt
                .iter()
                // .filter(|i| !starts_with_case_insensitive(i.name.as_str(), "reserved"))
                .cloned()
                .map(|mut i| {
                    i.name = i.name.trim().to_lowercase();
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

    println!("pub trait Runtime: crate::runtime::Ready + Sized {{");
    println!("    type Memory: crate::runtime::Memory;");
    println!("    type Arguments: crate::tuple::Tuple;");

    println!();
    println!("    fn new(mem: Self::Memory, cs: &CriticalSection) -> (Self, Self::Arguments);");
    println!();
    println!("    fn snapshot(&mut self, cs: &CriticalSection);");
    println!();
    println!("    fn idle(&self);");
    println!();
    println!("    fn wake(&mut self);");
    println!();
    println!("    fn shutdown(&self);");
    for i in &interrupts {
        if !i.name.starts_with("reserved") {
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
    }
    println!("}}");

    println!();
    println!("mod interrupts {{");
    for int in &interrupts {
        if int.name.starts_with("reserved") {
            println!("    #[doc(hidden)]");
            println!("    #[export_name = \"__vector_{}\"]", int.value);
            println!(
                "    unsafe extern \"avr-interrupt\" fn __vector_{}() {{}}",
                int.value
            );
        }
    }
    println!("}}");

    println!();
    println!("#[repr(C)]");
    println!("pub struct RawRuntime {{");
    println!("    pub data: *mut (),");
    println!("}}");
    println!();
    println!("unsafe impl Sync for RawRuntime {{}}");
    println!();
    println!("impl RawRuntime {{");
    println!("    #[inline(always)]");
    println!("    pub const fn uninit() -> Self {{");
    println!("        Self {{");
    println!("            data: 0 as *mut (),");
    println!("        }}");
    println!("    }}");
    println!();
    println!("    #[inline(always)]");
    println!("    pub const fn new<R: Runtime>(runtime: &R) -> Self {{");
    println!("        Self {{");
    println!("            data: runtime as *const R as *const () as *mut (),");
    println!("        }}");
    println!("    }}");
    println!();
    println!("    #[doc(hidden)]");
    println!("    #[inline(always)]");
    println!("    pub unsafe fn from_ptr<'a>(p: *const ()) -> &'a Self {{");
    println!("        &*(p as *const Self)");
    println!("    }}");
    println!("}}");

    Ok(())
}
