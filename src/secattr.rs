use std::ffi::c_void;

use winapi::{
    shared::{
        minwindef::{BOOL, BYTE, DWORD, LPVOID},
        sddl::ConvertSidToStringSidA,
    },
    um::{
        accctrl::{
            ACCESS_MODE, EXPLICIT_ACCESS_W, NO_INHERITANCE, SET_ACCESS, TRUSTEE_FORM,
            TRUSTEE_IS_GROUP, TRUSTEE_IS_SID, TRUSTEE_TYPE, TRUSTEE_W,
        },
        aclapi::SetEntriesInAclW,
        minwinbase::SECURITY_ATTRIBUTES,
        securitybaseapi::{
            AllocateAndInitializeSid, FreeSid, InitializeSecurityDescriptor,
            IsValidSecurityDescriptor, SetSecurityDescriptorDacl,
        },
        winbase::LocalFree,
        winnt::{
            ACL, KEY_ALL_ACCESS, SECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_REVISION,
            SECURITY_LOCAL_RID, SECURITY_LOCAL_SID_AUTHORITY, SECURITY_WORLD_RID,
            SECURITY_WORLD_SID_AUTHORITY, SID, SID_IDENTIFIER_AUTHORITY,
        },
    },
};

#[repr(transparent)]
pub struct SecurityAttributes {
    pub(crate) attrs: SECURITY_ATTRIBUTES,
}

impl SecurityAttributes {
    pub fn new(
        security_descriptor: &mut SecurityDescriptor,
        inherit_handle: bool,
    ) -> SecurityAttributes {
        SecurityAttributes {
            attrs: SECURITY_ATTRIBUTES {
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: security_descriptor as *mut _ as *mut _,
                bInheritHandle: inherit_handle as BOOL,
            },
        }
    }
}

impl Default for SecurityAttributes {
    fn default() -> Self {
        SecurityAttributes {
            attrs: SECURITY_ATTRIBUTES {
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: std::ptr::null_mut(),
                bInheritHandle: false as BOOL,
            },
        }
    }
}

pub struct Acl(*mut ACL);

impl Acl {
    pub fn new(explicit_accesses: &mut [ExplicitAccess]) -> std::io::Result<Acl> {
        let mut acl = std::ptr::null_mut();
        if unsafe { SetEntriesInAclW(0, std::ptr::null_mut(), std::ptr::null_mut(), &mut acl) } != 0
        {
            return Err(std::io::Error::last_os_error());
        }

        return Ok(Acl(acl));
    }
}

#[test]
fn test_acl() {
    SecurityDescriptor::world().unwrap();
}

#[repr(transparent)]
pub struct SecurityDescriptor(SECURITY_DESCRIPTOR);

impl SecurityDescriptor {
    pub fn new() -> std::io::Result<SecurityDescriptor> {
        let mut desc: SECURITY_DESCRIPTOR = unsafe { std::mem::zeroed() };

        if unsafe {
            InitializeSecurityDescriptor(
                &mut desc as *mut _ as *mut _,
                SECURITY_DESCRIPTOR_REVISION,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error());
        }

        if unsafe { IsValidSecurityDescriptor(&mut desc as *mut _ as *mut _) } == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "IsValidSecurityDescriptor returned false",
            ));
        }

        Ok(SecurityDescriptor(desc))
    }

    pub fn set_security_descriptor(&mut self, acl: Acl) -> std::io::Result<()> {
        if unsafe { SetSecurityDescriptorDacl(&mut self.0 as *mut _ as *mut _, 1, acl.0, 0) } == 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }

    pub fn world() -> std::io::Result<SecurityDescriptor> {
        let mut sd = Self::new()?;
        let trustee = Trustee::world();
        // sd.set_group(&Sid::world(), false)?;
        log::debug!("Creating all access for world");
        let ea = ExplicitAccess::all_access(&trustee);

        log::debug!("Creating DACL");
        let acl = Acl::new(&mut [ea])?;

        log::debug!("Setting DACL");
        sd.set_security_descriptor(acl)?;

        log::debug!("Returning SD");
        Ok(sd)
    }
}

#[repr(transparent)]
pub struct Sid(*const SID);

#[repr(transparent)]
pub struct ExplicitAccess(EXPLICIT_ACCESS_W);
#[repr(transparent)]
pub struct Trustee(TRUSTEE_W);

impl ExplicitAccess {
    pub unsafe fn from_raw(
        access_permissions: DWORD,
        access_mode: ACCESS_MODE,
        inheritance: DWORD,
        trustee: TRUSTEE_W,
    ) -> ExplicitAccess {
        ExplicitAccess(EXPLICIT_ACCESS_W {
            grfAccessPermissions: access_permissions,
            grfAccessMode: access_mode,
            grfInheritance: inheritance,
            Trustee: trustee,
        })
    }

    pub fn all_access(trustee: &Trustee) -> ExplicitAccess {
        ExplicitAccess(EXPLICIT_ACCESS_W {
            grfAccessPermissions: KEY_ALL_ACCESS,
            grfAccessMode: SET_ACCESS,
            grfInheritance: NO_INHERITANCE,
            Trustee: trustee.0,
        })
    }
}

impl Trustee {
    pub unsafe fn from_raw_sid(form: TRUSTEE_FORM, ty: TRUSTEE_TYPE, sid: *mut SID) -> Trustee {
        let mut t: TRUSTEE_W = std::mem::zeroed();
        t.TrusteeForm = form;
        t.TrusteeType = ty;
        t.ptstrName = sid as *mut _;
        Trustee(t)
    }

    pub fn world() -> Trustee {
        unsafe { Self::from_raw_sid(TRUSTEE_IS_SID, TRUSTEE_IS_GROUP, Sid::world().0 as *mut _) }
    }
}

impl std::fmt::Display for Sid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ptr = std::ptr::null_mut();
        let result = unsafe { ConvertSidToStringSidA(self.0 as *mut _, &mut ptr) };
        if result == 0 {
            log::error!("{}", std::io::Error::last_os_error());
            return Err(std::fmt::Error);
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
        let result = write!(f, "{}", c_str.to_string_lossy());
        unsafe { LocalFree(ptr as *mut _) };
        result
    }
}

impl Drop for Sid {
    fn drop(&mut self) {
        unsafe { FreeSid(self.0 as *mut _) };
    }
}

impl Sid {
    pub unsafe fn from_raw(
        authority: *const SID_IDENTIFIER_AUTHORITY,
        auth_count: BYTE,
        subauth: [DWORD; 8],
    ) -> std::io::Result<Sid> {
        let mut ptr = std::ptr::null_mut();
        let result = AllocateAndInitializeSid(
            authority as *mut _,
            auth_count,
            subauth[0],
            subauth[1],
            subauth[2],
            subauth[3],
            subauth[4],
            subauth[5],
            subauth[6],
            subauth[7],
            &mut ptr,
        );

        if result == 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(Sid(ptr as *const _))
    }

    pub fn world() -> Sid {
        unsafe {
            Self::from_raw(
                SECURITY_WORLD_SID_AUTHORITY.as_ptr() as _,
                1,
                [SECURITY_WORLD_RID, 0, 0, 0, 0, 0, 0, 0],
            )
            .unwrap()
        }
    }

    pub fn local() -> Sid {
        unsafe {
            Self::from_raw(
                SECURITY_LOCAL_SID_AUTHORITY.as_ptr() as _,
                1,
                [SECURITY_LOCAL_RID, 0, 0, 0, 0, 0, 0, 0],
            )
            .unwrap()
        }
    }

    pub fn as_ptr(&self) -> *const SID {
        self.0
    }

    pub fn as_mut_ptr(&self) -> *mut SID {
        self.0 as _
    }
}
