use super::InfoModule;

pub struct UsersModule;
impl UsersModule {
    pub fn new() -> Self {
        Self
    }
}

impl InfoModule for UsersModule {
    fn key(&self) -> &'static str {
        "Users"
    }
    fn value(&self) -> anyhow::Result<String> {
        let users = logged_in_users();
        if users.is_empty() {
            return Ok("0".to_string());
        }
        let mut list = users;
        list.sort();
        let count = list.len();
        Ok(format!("{count} ({})", list.join(", ")))
    }
}

#[cfg(unix)]
fn logged_in_users() -> Vec<String> {
    use std::collections::HashSet;
    use std::ffi::CStr;

    let mut set = HashSet::new();
    unsafe {
        libc::setutxent();
        loop {
            let ut = libc::getutxent();
            if ut.is_null() {
                break;
            }
            if (*ut).ut_type == libc::USER_PROCESS as libc::c_short {
                let user_ptr = (*ut).ut_user.as_ptr();
                if !user_ptr.is_null() {
                    let name = CStr::from_ptr(user_ptr).to_string_lossy().to_string();
                    if !name.is_empty() {
                        set.insert(name);
                    }
                }
            }
        }
        libc::endutxent();
    }
    set.into_iter().collect()
}

#[cfg(not(unix))]
fn logged_in_users() -> Vec<String> {
    Vec::new()
}
