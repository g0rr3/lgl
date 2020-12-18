////////////////////////////////////////////////////////////////////////////////////
// Copyright (c) 2020 DasEtwas - All Rights Reserved                               /
//      Unauthorized copying of this file, via any medium is strictly prohibited   /
//      Proprietary and confidential                                               /
////////////////////////////////////////////////////////////////////////////////////

use std::{borrow::Cow, io};

use lazy_static::*;
use registry::{Cmd, DebugPrints, Registry};

#[allow(missing_copy_implementations)]
pub struct GlobalTypedGenerator;

impl super::Generator for GlobalTypedGenerator {
    fn write<W>(&self, registry: &Registry, dest: &mut W) -> io::Result<()>
    where W: io::Write {
        write_header(dest)?;
        write_metaloadfn(dest)?;
        write_type_aliases(registry, dest)?;
        write_enums(registry, dest)?;
        write_fns(registry, dest)?;
        write_fnptr_struct_def(dest)?;
        write_ptrs(registry, dest)?;
        write_fn_mods(registry, dest)?;
        write_panicking_fns(registry, dest)?;
        write_load_fn(registry, dest)?;
        Ok(())
    }
}

/// Creates a `__gl_imports` module which contains all the external symbols that we need for the
///  bindings.
fn write_header<W>(dest: &mut W) -> io::Result<()>
where W: io::Write {
    writeln!(
        dest,
        r#"
#![allow(unused_parens, non_snake_case, dead_code, non_upper_case_globals, unused_variables, dead_code)]
mod __gl_imports {{
    pub use std::mem;
    pub use std::os::raw;
}}"#
    )
}

/// Creates the metaloadfn function for fallbacks
fn write_metaloadfn<W>(dest: &mut W) -> io::Result<()>
where W: io::Write {
    writeln!(
        dest,
        r#"
#[inline(never)]
fn metaloadfn(loadfn: &mut dyn FnMut(&'static str) -> *const __gl_imports::raw::c_void,
              symbol: &'static str,
              fallbacks: &[&'static str]) -> *const __gl_imports::raw::c_void {{
    let mut ptr = loadfn(symbol);
    if ptr.is_null() {{
        for &sym in fallbacks {{
            ptr = loadfn(sym);
            if !ptr.is_null() {{ break; }}
        }}
    }}
    ptr
}}
"#
    )
}

/// Creates a `types` module which contains all the type aliases.
///
/// See also `generators::gen_types`.
fn write_type_aliases<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where W: io::Write {
    writeln!(
        dest,
        r#"
pub mod types {{
    #![allow(non_camel_case_types, non_snake_case, dead_code, missing_copy_implementations)]
    "#
    )?;

    super::gen_types(registry.api, dest)?;

    writeln!(
        dest,
        "
}}
    "
    )
}

/// Creates all the `<enum>` elements at the root of the bindings.
fn write_enums<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where W: io::Write {
    for enm in &registry.enums {
        super::gen_enum_item(enm, "types::", dest)?;
    }

    Ok(())
}

/// Creates the functions corresponding to the GL commands.
///
/// The function calls the corresponding function pointer stored in the `storage` module created
///  by `write_ptrs`.
fn write_fns<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where W: io::Write {
    for cmd in &registry.cmds {
        if let Some(v) = registry.aliases.get(&cmd.proto.ident) {
            writeln!(dest, "            /// Fallbacks: {}", v.join(", "))?;
        }

        writeln!(
            dest,
            "#[inline]
pub fn {name}({params}) -> {return_suffix} {{
unsafe {{\
{conversions}\
{initializers}\
{debug_string_initializer}
let func_retv = __gl_imports::mem::transmute::<_, extern \"system\" fn({typed_params}) -> {return_suffix_without_ret_vals}>(storage::{name}.f)({idents});\
{debug_string_print}
{ret}
}}
}}",
            name = cmd.proto.ident,
            params = super::gen_parameters(&get_rustified(&without_return_types_params(cmd)), true, true).join(", "),
            typed_params = super::gen_parameters(cmd, false, true).join(", "),
            return_suffix = add_return_types(cmd, (&*cmd.proto.ty).to_string()),
            return_suffix_without_ret_vals = &*cmd.proto.ty,
            idents = super::gen_parameters(cmd, true, false).join(", "),
            debug_string_initializer = match registry.debug_prints {
                DebugPrints::None => "".to_owned(),
                DebugPrints::FunctionCalls => {
                    let params = super::gen_parameters(cmd, true, false);
                    format!(
                        "\nlet debug_string = format!(\"gl{name}{para1}{{:?}}{para2}\", ({debug_idents}));",
                        name = cmd.proto.ident,
                        debug_idents = {
                            if params.len() != 0 {
                                if params.len() < 15 {
                                    let joined = params.join(", &");
                                    if !joined.contains("callback") {
                                        format!("&{}", joined)
                                    } else {
                                        "\"<callback function as parameter>\"".to_owned()
                                    }
                                } else {
                                    // because there is no debug impl for a tuple of size 16 and up
                                    "\"<too many arguments to display>\"".to_owned()
                                }
                            } else {
                                // causes debug formatter to display () (fits for function call without params)
                                "".to_owned()
                            }
                        },
                        para1 = if params.len() != 1 { "" } else { "(" },
                        para2 = if params.len() != 1 { "" } else { ")" },
                    )
                },
            },
            debug_string_print = match registry.debug_prints {
                DebugPrints::None => "".to_owned(),
                DebugPrints::FunctionCalls => {
                    // only print returned values if return type is not unit
                    if cmd.proto.ty != "()" {
                        format!("\nprintln!(\"{{}} -> {{:?}}\", debug_string, {ret});", ret = get_return_args(cmd),)
                    } else {
                        format!("\nprintln!(\"{{}}\", debug_string);",)
                    }
                },
            },
            conversions = get_conversions(cmd),
            initializers = get_initializers(cmd),
            ret = get_return_args(cmd),
        )?;
    }

    Ok(())
}

lazy_static! {
    // provides the gl type to be swapped out, the rust type for the gl type and the conversion function
    static ref RUSTIFY_MAP: std::collections::HashMap<String, (String, String)> = {
        let mut map = std::collections::HashMap::new();
        //map.insert("types::GLuint".to_string(), ("u32".to_string(), "".to_string()));
        map.insert(
            "*const types::GLchar".to_string(),
            ("&str".to_string(),
            r#"let {param}_c_string = std::ffi::CString::new({param}.as_bytes()).expect("Failed to create CString in GL Call {cmd}");
let {param} = {param}_c_string.as_ptr();"#.to_string())
        );
        map.insert(
            "*const *const types::GLchar".to_string(),
            ("Vec<&str>".to_string(),
            r#"let {param}_c_string_vec = {param}.iter().map(|s| std::ffi::CString::new(s.as_bytes()).expect("Failed to create CString in GL Call {cmd}")).collect::<Vec<std::ffi::CString>>();
let {param}_vec = {param}_c_string_vec.iter().map(|c_string| c_string.as_ptr()).collect::<Vec<*const i8>>();
let {param} = {param}_vec.as_ptr();"#.to_string()));
        map.insert(
            "*const types::GLint".to_string(),
            ("&[types::GLint]".to_string(),
            r#"let {param} = {param}.as_ptr();"#.to_string()));
        map.insert(
            "*const types::GLfloat".to_string(),
            ("&[types::GLfloat]".to_string(),
            r#"let {param} = {param}.as_ptr();"#.to_string()));
        map.insert(
            "*const types::GLdouble".to_string(),
            ("&[types::GLdouble]".to_string(),
            r#"let {param} = {param}.as_ptr();"#.to_string()));
        map.insert(
            "*const types::GLuint".to_string(),
            ("&[types::GLuint]".to_string(),
            r#"let {param} = {param}.as_ptr();"#.to_string()));
        map.insert(
            "*const types::GLenum".to_string(),
            ("&[types::GLenum]".to_string(),
            r#"let {param} = {param}.as_ptr();"#.to_string()));
        map
    };

    // types (mutable ptrs) which are returned and do not require prior initialization
    // provides the gl type to be returned, the return type and the initialized value to pass on
    static ref RETURN_TYPES: std::collections::HashMap<String, (String, String)> = {
        let mut map = std::collections::HashMap::new();
        map.insert(
            "*mut types::GLsizei".to_string(),
            ("types::GLsizei".to_string(),
            r#"let mut {param}_ret = 0;
let {param} = &mut {param}_ret;"#.to_string())
        );
        map.insert(
            "*mut types::GLint".to_string(),
            ("types::GLint".to_string(),
            r#"// assuming that this pointer points to only one value
let mut {param}_ret = 0;
let {param} = &mut {param}_ret;"#.to_string())
        );
        map.insert(
            "*mut types::GLuint".to_string(),
            ("types::GLuint".to_string(),
            r#"// assuming that this pointer points to only one value
let mut {param}_ret = 0;
let {param} = &mut {param}_ret;"#.to_string())
        );
        map
    };
}

fn get_return_args(cmd: &Cmd) -> String {
    let mut ret = String::from("(");
    let mut i = 0;

    if cmd.proto.ty != "()" {
        ret.push_str("func_retv");
        i += 1;
    }

    cmd.params.iter().for_each(|param| {
        if let Some(_) = RETURN_TYPES.get(&*param.ty) {
            if i != 0 {
                ret.push_str(", ");
            }
            // {param}_ret
            ret.push_str(&*param.ident);
            ret.push_str("_ret");
            i += 1;
        }
    });

    ret.push_str(")");

    ret
}

fn add_return_types(cmd: &Cmd, ty: String) -> String {
    if ty == "()" {
        let mut ret = String::from("(");
        let mut i = 0;
        cmd.params.iter().for_each(|param| {
            if let Some((ty, _)) = RETURN_TYPES.get(&*param.ty) {
                if i != 0 {
                    ret.push_str(", ");
                }

                ret.push_str(&ty);
                i += 1;
            }
        });
        ret.push_str(")");
        ret
    } else {
        let mut ret = String::from("(");
        ret.push_str(&ty);
        cmd.params.iter().for_each(|param| {
            if let Some((ty, _)) = RETURN_TYPES.get(&*param.ty) {
                ret.push_str(", ");
                ret.push_str(&ty);
            }
        });
        ret.push_str(")");
        ret
    }
}

fn get_initializers(cmd: &Cmd) -> String {
    let mut initializers = String::from("\n");
    cmd.params.iter().for_each(|param| {
        if let Some((_, init_func)) = RETURN_TYPES.get(&*param.ty) {
            initializers.push_str("                    ");
            initializers.push_str(&init_func.replace("{param}", &*param.ident).replace("\n", "\n                    "));
            initializers.push_str("\n");
        }
    });
    if initializers.len() > 1 {
        initializers
    } else {
        String::new()
    }
}

fn is_return_type(ty: &str) -> bool {
    RETURN_TYPES.get(ty).is_some()
}

fn without_return_types_params(cmd: &Cmd) -> Cmd {
    let mut new = cmd.clone();
    new.params = new.params.clone();
    //new.params.drain_filter(|param| !is_return_type(&*param.ty));
    {
        let mut i = 0;
        while i != new.params.len() {
            if is_return_type(&*new.params[i].ty) {
                new.params.remove(i);
            } else {
                i += 1;
            }
        }
    }
    new
}

fn get_conversions(cmd: &Cmd) -> String {
    let mut conversions = String::from("\n");
    cmd.params.iter().for_each(|param| {
        if let Some((_, conv_func)) = RUSTIFY_MAP.get(&*param.ty) {
            conversions.push_str("                    ");
            conversions.push_str(&conv_func.replace("{param}", &*param.ident).replace("{cmd}", &*cmd.proto.ident).replace("\n", "\n                    "));
            conversions.push_str("\n");
        }
    });
    if conversions.len() > 1 {
        conversions
    } else {
        String::new()
    }
}

fn get_rustified(cmd: &Cmd) -> Cmd {
    let mut new = cmd.clone();
    new.params.iter_mut().for_each(|param| {
        if let Some((type_, _)) = RUSTIFY_MAP.get(&*param.ty) {
            param.ty = Cow::Borrowed(type_.as_str());
        }
    });
    new
}

/// Creates a `FnPtr` structure which contains the store for a single binding.
fn write_fnptr_struct_def<W>(dest: &mut W) -> io::Result<()>
where W: io::Write {
    writeln!(
        dest,
        "
#[allow(missing_copy_implementations)]
pub struct FnPtr {{
    /// The function pointer that will be used when calling the function.
    f: *const __gl_imports::raw::c_void,
    /// True if the pointer points to a real function, false if points to a `panic!` fn.
    is_loaded: bool,
}}

impl FnPtr {{
    /// Creates a `FnPtr` from a load attempt.
    pub fn new(ptr: *const __gl_imports::raw::c_void) -> FnPtr {{
        if ptr.is_null() {{
            FnPtr {{ f: missing_fn_panic as *const __gl_imports::raw::c_void, is_loaded: false }}
        }} else {{
            FnPtr {{ f: ptr, is_loaded: true }}
        }}
    }}
}}
    "
    )
}

/// Creates a `storage` module which contains a static `FnPtr` per GL command in the registry.
fn write_ptrs<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where W: io::Write {
    writeln!(
        dest,
        "mod storage {{
            #![allow(non_snake_case)]
            #![allow(non_upper_case_globals)]
            use super::__gl_imports::raw;
            use super::FnPtr;"
    )?;

    for c in &registry.cmds {
        writeln!(
            dest,
            "pub static mut {name}: FnPtr = FnPtr {{
                f: super::missing_fn_panic as *const raw::c_void,
                is_loaded: false
            }};",
            name = c.proto.ident
        )?;
    }

    writeln!(dest, "}}")
}

/// Creates one module for each GL command.
///
/// Each module contains `is_loaded` and `load_with` which interact with the `storage` module
///  created by `write_ptrs`.
fn write_fn_mods<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where W: io::Write {
    for c in &registry.cmds {
        let fallbacks = match registry.aliases.get(&c.proto.ident) {
            Some(v) => {
                let names = v.iter().map(|name| format!("\"{}\"", super::gen_symbol_name(registry.api, &name[..]))).collect::<Vec<_>>();
                format!("&[{}]", names.join(", "))
            },
            None => "&[]".to_string(),
        };
        let fnname = &c.proto.ident[..];
        let symbol = super::gen_symbol_name(registry.api, &c.proto.ident[..]);
        let symbol = &symbol[..];

        writeln!(
            dest,
            r##"
            #[allow(non_snake_case)]
            pub mod {fnname} {{
                use super::{{storage, metaloadfn}};
                use super::__gl_imports::raw;
                use super::FnPtr;

                #[inline]
                #[allow(dead_code)]
                pub fn is_loaded() -> bool {{
                    unsafe {{ storage::{fnname}.is_loaded }}
                }}

                #[allow(dead_code)]
                pub fn load_with<F>(mut loadfn: F) where F: FnMut(&'static str) -> *const raw::c_void {{
                    unsafe {{
                        storage::{fnname} = FnPtr::new(metaloadfn(&mut loadfn, "{symbol}", {fallbacks}))
                    }}
                }}
            }}
        "##,
            fnname = fnname,
            fallbacks = fallbacks,
            symbol = symbol
        )?;
    }

    Ok(())
}

/// Creates a `missing_fn_panic` function.
///
/// This function is the mock that is called if the real function could not be called.
fn write_panicking_fns<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where W: io::Write {
    writeln!(
        dest,
        "#[inline(never)]
        fn missing_fn_panic() -> ! {{
            panic!(\"{api} function was not loaded\")
        }}
        ",
        api = registry.api
    )
}

/// Creates the `load_with` function.
///
/// The function calls `load_with` in each module created by `write_fn_mods`.
fn write_load_fn<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where W: io::Write {
    writeln!(
        dest,
        "
        /// Load each OpenGL symbol using a custom load function. This allows for the
        /// use of functions like `glfwGetProcAddress` or `SDL_GL_GetProcAddress`.
        /// ~~~ignore
        /// gl::load_with(|s| glfw.get_proc_address(s));
        /// ~~~
        #[allow(dead_code)]
        pub fn load_with<F>(mut loadfn: F) where F: FnMut(&'static str) -> *const __gl_imports::raw::c_void {{
    "
    )?;

    for c in &registry.cmds {
        writeln!(dest, "{cmd_name}::load_with(&mut loadfn);", cmd_name = &c.proto.ident[..])?;
    }

    writeln!(
        dest,
        "
        }}
    "
    )
}
