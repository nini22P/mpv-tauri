use libmpv_sys as sys;
use std::ffi::{c_char, c_void, CStr};
use std::marker::PhantomData;
use std::ptr;

use crate::libmpv;

pub struct OpenGLInitParams<T: Send + 'static> {
    pub get_proc_address: fn(ctx: &T, name: &str) -> *mut c_void,
    pub user_context: T,
}

pub enum RenderParam<T: Send + 'static> {
    ApiTypeOpenGL,
    InitParams(OpenGLInitParams<T>),
}

struct TrampolinePayload<T: Send + 'static> {
    user_fn: fn(ctx: &T, name: &str) -> *mut c_void,
    user_ctx: T,
}

unsafe extern "C" fn get_proc_address_trampoline<T: Send + 'static>(
    ctx: *mut c_void,
    name: *const c_char,
) -> *mut c_void {
    if ctx.is_null() || name.is_null() {
        return ptr::null_mut();
    }
    let payload = &*(ctx as *const TrampolinePayload<T>);
    let symbol = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    (payload.user_fn)(&payload.user_ctx, symbol)
}

struct UpdateCallbackData {
    callback: Box<dyn FnMut() + Send>,
}

extern "C" fn update_callback_c(data: *mut c_void) {
    if data.is_null() {
        return;
    }
    let callback_data = unsafe { &mut *(data as *mut UpdateCallbackData) };
    (callback_data.callback)();
}

pub struct RenderContext<T: Send + 'static> {
    pub ctx: *mut sys::mpv_render_context,
    update_callback_data: *mut UpdateCallbackData,
    _trampoline_payload_owner: Option<Box<TrampolinePayload<T>>>,

    _opengl_init_params_owner: Option<Box<sys::mpv_opengl_init_params>>,
    _phantom: PhantomData<T>,
}

unsafe impl<T: Send + 'static> Send for RenderContext<T> {}
unsafe impl<T: Send + 'static> Sync for RenderContext<T> {}

impl<T: Send + 'static> RenderContext<T> {
    pub fn new(mpv: &libmpv::Mpv, params: Vec<RenderParam<T>>) -> Result<Self, String> {
        let mut trampoline_payload_owner: Option<Box<TrampolinePayload<T>>> = None;

        let mut opengl_init_params_owner: Option<Box<sys::mpv_opengl_init_params>> = None;
        let mut mpv_params: Vec<sys::mpv_render_param> = Vec::new();

        for param in params {
            match param {
                RenderParam::ApiTypeOpenGL => {
                    mpv_params.push(sys::mpv_render_param {
                        type_: sys::mpv_render_param_type_MPV_RENDER_PARAM_API_TYPE,
                        data: sys::MPV_RENDER_API_TYPE_OPENGL.as_ptr() as *mut _,
                    });
                }
                RenderParam::InitParams(gl_params) => {
                    let payload = Box::new(TrampolinePayload {
                        user_fn: gl_params.get_proc_address,
                        user_ctx: gl_params.user_context,
                    });
                    let payload_ptr = &*payload as *const _ as *mut c_void;

                    let opengl_init_params = Box::new(sys::mpv_opengl_init_params {
                        get_proc_address: Some(get_proc_address_trampoline::<T>),
                        get_proc_address_ctx: payload_ptr,
                    });

                    let params_ptr = &*opengl_init_params as *const _ as *mut c_void;

                    mpv_params.push(sys::mpv_render_param {
                        type_: sys::mpv_render_param_type_MPV_RENDER_PARAM_OPENGL_INIT_PARAMS,

                        data: params_ptr,
                    });

                    opengl_init_params_owner = Some(opengl_init_params);

                    trampoline_payload_owner = Some(payload);
                }
            }
        }

        mpv_params.push(sys::mpv_render_param {
            type_: sys::mpv_render_param_type_MPV_RENDER_PARAM_INVALID,
            data: ptr::null_mut(),
        });

        let mut ctx: *mut sys::mpv_render_context = ptr::null_mut();
        let err = unsafe {
            sys::mpv_render_context_create(&mut ctx, mpv.handle, mpv_params.as_mut_ptr())
        };
        if err < 0 {
            return Err(format!("Failed to create render context: {}", err));
        }

        Ok(Self {
            ctx,
            update_callback_data: ptr::null_mut(),
            _trampoline_payload_owner: trampoline_payload_owner,
            _opengl_init_params_owner: opengl_init_params_owner,
            _phantom: PhantomData,
        })
    }

    pub fn set_update_callback<F>(&mut self, callback: F)
    where
        F: FnMut() + Send + 'static,
    {
        if !self.update_callback_data.is_null() {
            let _ = unsafe { Box::from_raw(self.update_callback_data) };
        }
        let callback_data = Box::new(UpdateCallbackData {
            callback: Box::new(callback),
        });
        let data_ptr = Box::into_raw(callback_data);
        self.update_callback_data = data_ptr;
        unsafe {
            sys::mpv_render_context_set_update_callback(
                self.ctx,
                Some(update_callback_c),
                data_ptr as *mut c_void,
            );
        }
    }

    pub fn render(&self, width: i32, height: i32) -> Result<(), String> {
        let fbo = sys::mpv_opengl_fbo {
            fbo: 0,
            w: width,
            h: height,
            internal_format: 0,
        };
        let mut flip_y: i32 = 1;
        let mut params = [
            sys::mpv_render_param {
                type_: sys::mpv_render_param_type_MPV_RENDER_PARAM_OPENGL_FBO,
                data: &fbo as *const _ as *mut c_void,
            },
            sys::mpv_render_param {
                type_: sys::mpv_render_param_type_MPV_RENDER_PARAM_FLIP_Y,
                data: &mut flip_y as *mut _ as *mut c_void,
            },
            sys::mpv_render_param {
                type_: sys::mpv_render_param_type_MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];
        let err = unsafe { sys::mpv_render_context_render(self.ctx, params.as_mut_ptr()) };
        if err < 0 {
            return Err(format!("Failed to render: {}", err));
        }
        Ok(())
    }
}

impl<T: Send + 'static> Drop for RenderContext<T> {
    fn drop(&mut self) {
        unsafe {
            if !self.ctx.is_null() {
                sys::mpv_render_context_set_update_callback(self.ctx, None, ptr::null_mut());
            }
        }

        if !self.update_callback_data.is_null() {
            let _ = unsafe { Box::from_raw(self.update_callback_data) };
            self.update_callback_data = ptr::null_mut();
        }
        if !self.ctx.is_null() {
            unsafe { sys::mpv_render_context_free(self.ctx) };
        }
    }
}
