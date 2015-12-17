macro_rules! decl_objc_callback {
    ($name:ident, $cbs_name:ident) => (
		// this code is pretty much a rip off of
		// https://github.com/SSheldon/rust-objc-foundation/blob/master/examples/custom_class.rs

		// would be nice to use some mangled ident names here base on $name,
		// (and avoid the need for $cbs_name)
		// but concat_idents! doesn't work in the cases that I want.
		enum $name {};
		unsafe impl Message for $name { }

		// SO.. some explanation is in order here.  We want to allow closure callbacks that
		// can modify their environment.  But we can't keep them on the $name object because
		// that is really just a stateless proxy for the objc object.  So we store them
		// as numeric pointers values in "ivar" fields on that object.  But, if we store a pointer to the
		// closure object, we'll run into issues with thin/fat pointer conversions (because
		// closure objects are trait objects and thus fat pointers).  So we wrap the closure in
		// another boxed object ($cbs_name), which, since it doesn't use traits, is actually a
		// regular "thin" pointer, and store THAT pointer in the ivar.  But...so...oy.
		struct $cbs_name {
			cb: Box<Fn() -> ()>
		}

		impl $name {
			fn from(cb:Box<Fn() -> ()>) -> Id<$name> {
				let cbs = $cbs_name {
					cb: cb
				};
				let bcbs = Box::new(cbs);

				let ptr = Box::into_raw(bcbs);
				let ptr = ptr as *mut libc::c_void as u64;
				println!("{}", ptr);
				let mut oid = $name::new();
				(*oid).setptr(ptr);
				oid
			}

			fn setptr(&mut self, uptr: u64) {
		        unsafe {
		            let obj =  &mut *(self as *mut _ as *mut Object);
					println!("setting the ptr: {}", uptr);
		            obj.set_ivar("_cbptr", uptr);
		        }
		    }
		}

		// TODO: Drop for $name doesn't get called, probably because objc manages the memory and
		// releases it for us.  so we leak the boxed callback right now.

		impl INSObject for $name {
			fn class() -> &'static Class {
				let cname = stringify!($name);

				let mut klass = Class::get(cname);
				if klass.is_none() {
					println!("registering class for {}", cname);
					let superclass = NSObject::class();
					let mut decl = ClassDecl::new(superclass, &cname).unwrap();
					decl.add_ivar::<u64>("_cbptr");

					extern fn $name(this: &Object, _cmd: Sel) {
						println!("callback, getting the pointer");
						unsafe {
							let pval:u64 = *this.get_ivar("_cbptr");
							let ptr = pval as *mut libc::c_void;
							let ptr = ptr as *mut $cbs_name;
							let bcbs:Box<$cbs_name> = Box::from_raw(ptr);
							{
								println!("cb test from cb");
								(*bcbs.cb)();
							}
							mem::forget(bcbs);
						}
					}

					unsafe {
						decl.add_method(sel!($name), $name as extern fn(&Object, Sel));
					}

					decl.register();
					klass = Class::get(cname);
				}
				klass.unwrap()
			}
		}
	);
}


#[test]
fn it_works() {
}
