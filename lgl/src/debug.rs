use gl::types::{GLchar, GLenum, GLsizei, GLuint};
use std::ffi::CString;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GLErrorSeverityLogLevel {
    DEBUG_SEVERITY_HIGH = 5,
    DEBUG_SEVERITY_MEDIUM = 4,
    DEBUG_SEVERITY_LOW = 3,
    DEBUG_SEVERITY_NOTIFICATION = 2,
    All = 1,
}

impl GLErrorSeverityLogLevel {
    fn from_gl(gl: GLenum) -> GLErrorSeverityLogLevel {
        match gl {
            gl::DEBUG_SEVERITY_HIGH => GLErrorSeverityLogLevel::DEBUG_SEVERITY_HIGH,
            gl::DEBUG_SEVERITY_MEDIUM => GLErrorSeverityLogLevel::DEBUG_SEVERITY_MEDIUM,
            gl::DEBUG_SEVERITY_LOW => GLErrorSeverityLogLevel::DEBUG_SEVERITY_LOW,
            gl::DEBUG_SEVERITY_NOTIFICATION => GLErrorSeverityLogLevel::DEBUG_SEVERITY_NOTIFICATION,
            _ => GLErrorSeverityLogLevel::All,
        }
    }
}

pub unsafe fn enable_gl_debug(log_level: GLErrorSeverityLogLevel) {
    // this is never freed, it leaks memory (1 byte)
    let log_level = Box::leak(Box::new(log_level));

    gl::DebugMessageCallback(callback, log_level as *mut _ as *mut _);

    assert!(GLErrorSeverityLogLevel::All < GLErrorSeverityLogLevel::DEBUG_SEVERITY_HIGH);

    let context_flags = 0;
    gl::GetIntegerv(gl::CONTEXT_FLAGS);

    if context_flags as u32 & gl::CONTEXT_FLAG_DEBUG_BIT == 0 {
        eprintln!("This is a non-debug OpenGL context which may not produce any debug output.");
        gl::Enable(gl::DEBUG_OUTPUT);
    }
}

extern "system" fn callback(
    source: GLenum,
    gltype: GLenum,
    id: GLuint,
    severity: GLenum,
    _length: GLsizei,
    message: *const GLchar,
    log_level: *mut core::ffi::c_void,
) {
    // return if the severity is not high enough
    if GLErrorSeverityLogLevel::from_gl(severity)
        < unsafe { *(log_level as *mut GLErrorSeverityLogLevel) }
    {
        return;
    }

    let source_string = match source {
        gl::DEBUG_SOURCE_API => "DEBUG_SOURCE_API",
        gl::DEBUG_SOURCE_APPLICATION => "DEBUG_SOURCE_APPLICATION",
        gl::DEBUG_SOURCE_OTHER => "DEBUG_SOURCE_OTHER",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "DEBUG_SOURCE_SHADER_COMPILER",
        gl::DEBUG_SOURCE_THIRD_PARTY => "DEBUG_SOURCE_THIRD_PARTY",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "DEBUG_SOURCE_WINDOW_SYSTEM",
        _ => "Unknown source",
    };

    let severity_string = match severity {
        gl::DEBUG_SEVERITY_HIGH => "HIGH",
        gl::DEBUG_SEVERITY_MEDIUM => "MEDIUM",
        gl::DEBUG_SEVERITY_LOW => "LOW",
        gl::DEBUG_SEVERITY_NOTIFICATION => "NOTIFICATION",
        _ => "Unknown severity",
    };

    let type_string = match gltype {
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "DEBUG_TYPE_DEPRECATED_BEHAVIOR",
        gl::DEBUG_TYPE_ERROR => "DEBUG_TYPE_ERROR",
        gl::DEBUG_TYPE_MARKER => "DEBUG_TYPE_MARKER",
        gl::DEBUG_TYPE_OTHER => "DEBUG_TYPE_OTHER",
        gl::DEBUG_TYPE_PERFORMANCE => "DEBUG_TYPE_PERFORMANCE",
        gl::DEBUG_TYPE_POP_GROUP => "DEBUG_TYPE_POP_GROUP",
        gl::DEBUG_TYPE_PORTABILITY => "DEBUG_TYPE_PORTABILITY",
        gl::DEBUG_TYPE_PUSH_GROUP => "DEBUG_TYPE_PUSH_GROUP",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "DEBUG_TYPE_UNDEFINED_BEHAVIOR",
        _ => "Unknown error type",
    };

    let message = unsafe { CString::from_raw(message as *mut _) };
    let message_string = message.to_str().unwrap_or("Failed to read debug message");

    {
        eprintln!(
            "OpenGL Debug Message
  Severity : {}
  Source   : {}
  ID       : {}
  GLType   : {}
  Message  : {}",
            severity_string,
            source_string,
            format!("0x{:8X}", id).replace(" ", "0"),
            type_string,
            message_string
        );
    }

    // opengl handles freeing the string
    std::mem::forget(message);
}
