use crate::{
    io,
    net::{SocketAddr, SocketAddrUnix, SocketAddrV4, SocketAddrV6},
};
#[cfg(any(linux_raw, all(libc, not(any(target_os = "ios", target_os = "macos")))))]
use bitflags::bitflags;
use io_lifetimes::{AsFd, BorrowedFd, OwnedFd};
use std::mem::{size_of, MaybeUninit};
use std::os::raw::c_int;
#[cfg(linux_raw)]
use std::os::raw::c_uint;
#[cfg(libc)]
use {
    crate::{negone_err, zero_ok},
    libc::{sockaddr_storage, socklen_t},
    unsafe_io::os::posish::{AsRawFd, FromRawFd},
};

/// `SOCK_*` constants for `socket`.
#[cfg(libc)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u32)]
#[non_exhaustive]
pub enum SocketType {
    /// `SOCK_STREAM`.
    Stream = libc::SOCK_STREAM as u32,

    /// `SOCK_DGRAM`.
    Datagram = libc::SOCK_DGRAM as u32,

    /// `SOCK_SEQPACKET`.
    SeqPacket = libc::SOCK_SEQPACKET as u32,

    /// `SOCK_RAW`.
    Raw = libc::SOCK_RAW as u32,

    /// `SOCK_RDM`.
    Rdm = libc::SOCK_RDM as u32,
}

/// `SOCK_*` constants for `socket`.
#[cfg(linux_raw)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u32)]
#[non_exhaustive]
pub enum SocketType {
    /// `SOCK_STREAM`.
    Stream = linux_raw_sys::general::SOCK_STREAM,

    /// `SOCK_DGRAM`.
    Datagram = linux_raw_sys::general::SOCK_DGRAM,

    /// `SOCK_SEQPACKET`.
    SeqPacket = linux_raw_sys::general::SOCK_SEQPACKET,

    /// `SOCK_RAW`.
    Raw = linux_raw_sys::general::SOCK_RAW,

    /// `SOCK_RDM`.
    Rdm = linux_raw_sys::general::SOCK_RDM,
}

/// `AF_*` constants.
#[cfg(libc)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u32)]
#[non_exhaustive]
pub enum AddressFamily {
    /// `AF_LOCAL`, aka `AF_UNIX`
    #[doc(alias = "Unix")]
    Local = libc::AF_LOCAL as u32,

    /// `AF_INET`
    Inet = libc::AF_INET as u32,

    /// `AF_INET6`
    Inet6 = libc::AF_INET6 as u32,

    /// `AF_NETLINK`
    #[cfg(not(any(
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd"
    )))]
    Netlink = libc::AF_NETLINK as u32,
}

/// `AF_*` constants.
#[cfg(linux_raw)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u32)]
#[non_exhaustive]
pub enum AddressFamily {
    /// `AF_LOCAL`, aka `AF_UNIX`
    #[doc(alias = "Unix")]
    Local = linux_raw_sys::general::AF_LOCAL,

    /// `AF_INET`
    Inet = linux_raw_sys::general::AF_INET,

    /// `AF_INET6`
    Inet6 = linux_raw_sys::general::AF_INET6,

    /// `AF_NETLINK`
    Netlink = linux_raw_sys::general::AF_NETLINK,
}

/// `IPPROTO_*`
#[cfg(libc)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(i32)]
#[non_exhaustive]
pub enum Protocol {
    /// `IPPROTO_IP`
    Ip = libc::IPPROTO_IP,
    /// `IPPROTO_ICMP`
    Icmp = libc::IPPROTO_ICMP,
    /// `IPPROTO_IGMP`
    Igmp = libc::IPPROTO_IGMP,
    /// `IPPROTO_IPIP`
    Ipip = libc::IPPROTO_IPIP,
    /// `IPPROTO_TCP`
    Tcp = libc::IPPROTO_TCP,
    /// `IPPROTO_EGP`
    Egp = libc::IPPROTO_EGP,
    /// `IPPROTO_PUP`
    Pup = libc::IPPROTO_PUP,
    /// `IPPROTO_UDP`
    Udp = libc::IPPROTO_UDP,
    /// `IPPROTO_IDP`
    Idp = libc::IPPROTO_IDP,
    /// `IPPROTO_TP`
    Tp = libc::IPPROTO_TP,
    /// `IPPROTO_DCCP`
    #[cfg(not(any(target_os = "ios", target_os = "macos")))]
    Dccp = libc::IPPROTO_DCCP,
    /// `IPPROTO_IPV6`
    Ipv6 = libc::IPPROTO_IPV6,
    /// `IPPROTO_RSVP`
    Rsvp = libc::IPPROTO_RSVP,
    /// `IPPROTO_GRE`
    Gre = libc::IPPROTO_GRE,
    /// `IPPROTO_ESP`
    Esp = libc::IPPROTO_ESP,
    /// `IPPROTO_AH`
    Ah = libc::IPPROTO_AH,
    /// `IPPROTO_MTP`
    #[cfg(not(target_os = "netbsd"))]
    Mtp = libc::IPPROTO_MTP,
    /// `IPPROTO_BEETPH`
    #[cfg(not(any(
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "freebsd"
    )))]
    Beetph = libc::IPPROTO_BEETPH,
    /// `IPPROTO_ENCAP`
    Encap = libc::IPPROTO_ENCAP,
    /// `IPPROTO_PIM`
    Pim = libc::IPPROTO_PIM,
    /// `IPPROTO_COMP`
    #[cfg(not(any(
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "freebsd"
    )))]
    Comp = libc::IPPROTO_COMP,
    /// `IPPROTO_SCTP`
    Sctp = libc::IPPROTO_SCTP,
    /// `IPPROTO_UDPLITE`
    #[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "netbsd")))]
    Udplite = libc::IPPROTO_UDPLITE,
    /// `IPPROTO_MPLS`
    #[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "netbsd")))]
    Mpls = libc::IPPROTO_MPLS,
    /// `IPPROTO_RAW`
    Raw = libc::IPPROTO_RAW,
    /// `IPPROTO_MPTCP`
    #[cfg(not(any(
        target_os = "android",
        target_os = "netbsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "freebsd",
        target_os = "emscripten",
        target_os = "fuchsia"
    )))]
    Mptcp = libc::IPPROTO_MPTCP,
}

/// `IPPROTO_*`
#[cfg(linux_raw)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u32)]
#[non_exhaustive]
pub enum Protocol {
    /// `IPPROTO_IP`
    Ip = linux_raw_sys::general::IPPROTO_IP as u32,
    /// `IPPROTO_ICMP`
    Icmp = linux_raw_sys::general::IPPROTO_ICMP as u32,
    /// `IPPROTO_IGMP`
    Igmp = linux_raw_sys::general::IPPROTO_IGMP as u32,
    /// `IPPROTO_IPIP`
    Ipip = linux_raw_sys::general::IPPROTO_IPIP as u32,
    /// `IPPROTO_TCP`
    Tcp = linux_raw_sys::general::IPPROTO_TCP as u32,
    /// `IPPROTO_EGP`
    Egp = linux_raw_sys::general::IPPROTO_EGP as u32,
    /// `IPPROTO_PUP`
    Pup = linux_raw_sys::general::IPPROTO_PUP as u32,
    /// `IPPROTO_UDP`
    Udp = linux_raw_sys::general::IPPROTO_UDP as u32,
    /// `IPPROTO_IDP`
    Idp = linux_raw_sys::general::IPPROTO_IDP as u32,
    /// `IPPROTO_TP`
    Tp = linux_raw_sys::v5_4::general::IPPROTO_TP as u32,
    /// `IPPROTO_DCCP`
    Dccp = linux_raw_sys::general::IPPROTO_DCCP as u32,
    /// `IPPROTO_IPV6`
    Ipv6 = linux_raw_sys::general::IPPROTO_IPV6 as u32,
    /// `IPPROTO_RSVP`
    Rsvp = linux_raw_sys::general::IPPROTO_RSVP as u32,
    /// `IPPROTO_GRE`
    Gre = linux_raw_sys::general::IPPROTO_GRE as u32,
    /// `IPPROTO_ESP`
    Esp = linux_raw_sys::general::IPPROTO_ESP as u32,
    /// `IPPROTO_AH`
    Ah = linux_raw_sys::general::IPPROTO_AH as u32,
    /// `IPPROTO_MTP`
    Mtp = linux_raw_sys::v5_4::general::IPPROTO_MTP as u32,
    /// `IPPROTO_BEETPH`
    Beetph = linux_raw_sys::general::IPPROTO_BEETPH as u32,
    /// `IPPROTO_ENCAP`
    Encap = linux_raw_sys::v5_4::general::IPPROTO_ENCAP as u32,
    /// `IPPROTO_PIM`
    Pim = linux_raw_sys::general::IPPROTO_PIM as u32,
    /// `IPPROTO_COMP`
    Comp = linux_raw_sys::general::IPPROTO_COMP as u32,
    /// `IPPROTO_SCTP`
    Sctp = linux_raw_sys::general::IPPROTO_SCTP as u32,
    /// `IPPROTO_UDPLITE`
    Udplite = linux_raw_sys::general::IPPROTO_UDPLITE as u32,
    /// `IPPROTO_MPLS`
    Mpls = linux_raw_sys::v5_4::general::IPPROTO_MPLS as u32,
    /// `IPPROTO_ETHERNET`
    Ethernet = linux_raw_sys::v5_11::general::IPPROTO_ETHERNET as u32,
    /// `IPPROTO_RAW`
    Raw = linux_raw_sys::general::IPPROTO_RAW as u32,
    /// `IPPROTO_MPTCP`
    Mptcp = linux_raw_sys::v5_11::general::IPPROTO_MPTCP as u32,
}

/// `SHUT_*`
#[cfg(libc)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum Shutdown {
    /// `SHUT_RD`
    Read = libc::SHUT_RD,
    /// `SHUT_WR`
    Write = libc::SHUT_WR,
    /// `SHUT_RDWR`
    ReadWrite = libc::SHUT_RDWR,
}

/// `SHUT_*`
#[cfg(linux_raw)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u32)]
pub enum Shutdown {
    /// `SHUT_RD`
    Read = linux_raw_sys::general::SHUT_RD,
    /// `SHUT_WR`
    Write = linux_raw_sys::general::SHUT_WR,
    /// `SHUT_RDWR`
    ReadWrite = linux_raw_sys::general::SHUT_RDWR,
}

#[cfg(all(libc, not(any(target_os = "ios", target_os = "macos"))))]
bitflags! {
    /// `SOCK_*` constants for `accept`.
    pub struct AcceptFlags: c_int {
        /// `SOCK_NONBLOCK`
        const NONBLOCK = libc::SOCK_NONBLOCK;
        /// `SOCK_CLOEXEC`
        const CLOEXEC = libc::SOCK_CLOEXEC;
    }
}

#[cfg(linux_raw)]
bitflags! {
    /// `SOCK_*` constants for `accept`.
    pub struct AcceptFlags: c_uint {
        /// `SOCK_NONBLOCK`
        const NONBLOCK = linux_raw_sys::general::O_NONBLOCK;
        /// `SOCK_CLOEXEC`
        const CLOEXEC = linux_raw_sys::general::O_CLOEXEC;
    }
}

/// `socket(domain, type_, protocol)`
#[inline]
pub fn socket(domain: AddressFamily, type_: SocketType, protocol: Protocol) -> io::Result<OwnedFd> {
    _socket(domain, type_, protocol)
}

#[cfg(libc)]
fn _socket(domain: AddressFamily, type_: SocketType, protocol: Protocol) -> io::Result<OwnedFd> {
    unsafe {
        let raw_fd = negone_err(libc::socket(
            domain as c_int,
            type_ as c_int,
            protocol as c_int,
        ))?;
        Ok(OwnedFd::from_raw_fd(raw_fd))
    }
}

#[cfg(linux_raw)]
fn _socket(domain: AddressFamily, type_: SocketType, protocol: Protocol) -> io::Result<OwnedFd> {
    crate::linux_raw::socket(domain as c_uint, type_ as c_uint, protocol as c_uint)
}

/// `bind(sockfd, addr, sizeof(struct sockaddr_un))`
#[inline]
pub fn bind_un<'f, Fd: AsFd<'f>>(sockfd: Fd, addr: &SocketAddrUnix) -> io::Result<()> {
    let sockfd = sockfd.as_fd();
    _bind_un(sockfd, addr)
}

#[cfg(libc)]
fn _bind_un(sockfd: BorrowedFd<'_>, addr: &SocketAddrUnix) -> io::Result<()> {
    unsafe {
        zero_ok(libc::bind(
            sockfd.as_raw_fd(),
            addr as *const _ as *const _,
            size_of::<SocketAddrUnix>() as socklen_t,
        ))
    }
}

#[cfg(linux_raw)]
fn _bind_un(sockfd: BorrowedFd<'_>, addr: &SocketAddrUnix) -> io::Result<()> {
    crate::linux_raw::bind_un(sockfd, addr)
}

/// `bind(sockfd, addr, sizeof(struct sockaddr_in))`
#[inline]
pub fn bind_in<'f, Fd: AsFd<'f>>(sockfd: Fd, addr: &SocketAddrV4) -> io::Result<()> {
    let sockfd = sockfd.as_fd();
    _bind_in(sockfd, addr)
}

#[cfg(libc)]
fn _bind_in(sockfd: BorrowedFd<'_>, addr: &SocketAddrV4) -> io::Result<()> {
    unsafe {
        zero_ok(libc::bind(
            sockfd.as_raw_fd(),
            addr as *const _ as *const _,
            size_of::<SocketAddrV4>() as socklen_t,
        ))
    }
}

#[cfg(linux_raw)]
fn _bind_in(sockfd: BorrowedFd<'_>, addr: &SocketAddrV4) -> io::Result<()> {
    crate::linux_raw::bind_in(sockfd, addr)
}

/// `bind(sockfd, addr, sizeof(struct sockaddr_in6))`
#[inline]
pub fn bind_in6<'f, Fd: AsFd<'f>>(sockfd: Fd, addr: &SocketAddrV6) -> io::Result<()> {
    let sockfd = sockfd.as_fd();
    _bind_in6(sockfd, addr)
}

#[cfg(libc)]
fn _bind_in6(sockfd: BorrowedFd<'_>, addr: &SocketAddrV6) -> io::Result<()> {
    unsafe {
        zero_ok(libc::bind(
            sockfd.as_raw_fd(),
            addr as *const _ as *const _,
            size_of::<SocketAddrV6>() as socklen_t,
        ))
    }
}

#[cfg(linux_raw)]
fn _bind_in6(sockfd: BorrowedFd<'_>, addr: &SocketAddrV6) -> io::Result<()> {
    crate::linux_raw::bind_in6(sockfd, addr)
}

/// `connect(sockfd, addr, sizeof(struct sockaddr_un))`
#[inline]
pub fn connect_un<'f, Fd: AsFd<'f>>(sockfd: Fd, addr: &SocketAddrUnix) -> io::Result<()> {
    let sockfd = sockfd.as_fd();
    _connect_un(sockfd, addr)
}

#[cfg(libc)]
fn _connect_un(sockfd: BorrowedFd<'_>, addr: &SocketAddrUnix) -> io::Result<()> {
    unsafe {
        zero_ok(libc::connect(
            sockfd.as_raw_fd(),
            addr as *const _ as *const _,
            size_of::<SocketAddrUnix>() as socklen_t,
        ))
    }
}

#[cfg(linux_raw)]
fn _connect_un(sockfd: BorrowedFd<'_>, addr: &SocketAddrUnix) -> io::Result<()> {
    crate::linux_raw::connect_un(sockfd, addr)
}

/// `connect(sockfd, addr, sizeof(struct sockaddr_in))`
#[inline]
pub fn connect_in<'f, Fd: AsFd<'f>>(sockfd: Fd, addr: &SocketAddrV4) -> io::Result<()> {
    let sockfd = sockfd.as_fd();
    _connect_in(sockfd, addr)
}

#[cfg(libc)]
fn _connect_in(sockfd: BorrowedFd<'_>, addr: &SocketAddrV4) -> io::Result<()> {
    unsafe {
        zero_ok(libc::connect(
            sockfd.as_raw_fd(),
            addr as *const _ as *const _,
            size_of::<SocketAddrV4>() as socklen_t,
        ))
    }
}

#[cfg(linux_raw)]
fn _connect_in(sockfd: BorrowedFd<'_>, addr: &SocketAddrV4) -> io::Result<()> {
    crate::linux_raw::connect_in(sockfd, addr)
}

/// `connect(sockfd, addr, sizeof(struct sockaddr_in6))`
#[inline]
pub fn connect_in6<'f, Fd: AsFd<'f>>(sockfd: Fd, addr: &SocketAddrV6) -> io::Result<()> {
    let sockfd = sockfd.as_fd();
    _connect_in6(sockfd, addr)
}

#[cfg(libc)]
fn _connect_in6(sockfd: BorrowedFd<'_>, addr: &SocketAddrV6) -> io::Result<()> {
    unsafe {
        zero_ok(libc::connect(
            sockfd.as_raw_fd(),
            addr as *const _ as *const _,
            size_of::<SocketAddrV6>() as socklen_t,
        ))
    }
}

#[cfg(linux_raw)]
fn _connect_in6(sockfd: BorrowedFd<'_>, addr: &SocketAddrV6) -> io::Result<()> {
    crate::linux_raw::connect_in6(sockfd, addr)
}

/// `listen(fd, backlog)`
#[inline]
pub fn listen<'f, Fd: AsFd<'f>>(sockfd: Fd, backlog: c_int) -> io::Result<()> {
    let sockfd = sockfd.as_fd();
    _listen(sockfd, backlog)
}

#[cfg(libc)]
fn _listen(sockfd: BorrowedFd<'_>, backlog: c_int) -> io::Result<()> {
    unsafe { zero_ok(libc::listen(sockfd.as_raw_fd(), backlog)) }
}

#[cfg(linux_raw)]
#[inline]
fn _listen(sockfd: BorrowedFd<'_>, backlog: c_int) -> io::Result<()> {
    crate::linux_raw::listen(sockfd, backlog)
}

/// `accept(fd, addr, len)`
#[cfg(any(target_os = "ios", target_os = "macos"))]
#[inline]
pub fn accept<'f, Fd: AsFd<'f>>(sockfd: Fd) -> io::Result<(OwnedFd, SocketAddr)> {
    let sockfd = sockfd.as_fd();
    _accept(sockfd)
}

#[cfg(all(libc, any(target_os = "ios", target_os = "macos")))]
fn _accept(sockfd: BorrowedFd<'_>) -> io::Result<(OwnedFd, SocketAddr)> {
    unsafe {
        let mut storage = MaybeUninit::<sockaddr_storage>::uninit();
        let mut len = size_of::<sockaddr_storage>() as socklen_t;
        let raw_fd = negone_err(libc::accept(
            sockfd.as_raw_fd(),
            storage.as_mut_ptr() as *mut _,
            &mut len,
        ))?;
        let owned_fd = OwnedFd::from_raw_fd(raw_fd);
        let storage = storage.assume_init();
        let addr = match i32::from(storage.ss_family) {
            libc::AF_INET => {
                assert!(len as usize >= size_of::<SocketAddrV4>());
                SocketAddr::V4((*(&storage as *const _ as *const SocketAddrV4)).clone())
            }
            libc::AF_INET6 => {
                assert!(len as usize >= size_of::<SocketAddrV6>());
                SocketAddr::V6((*(&storage as *const _ as *const SocketAddrV6)).clone())
            }
            libc::AF_LOCAL => {
                assert!(len as usize >= size_of::<SocketAddrUnix>());
                SocketAddr::Unix((*(&storage as *const _ as *const SocketAddrUnix)).clone())
            }
            _ => panic!(),
        };
        Ok((owned_fd, addr))
    }
}

/// `accept4(fd, addr, len, flags)`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
#[inline]
pub fn accept4<'f, Fd: AsFd<'f>>(
    sockfd: Fd,
    flags: AcceptFlags,
) -> io::Result<(OwnedFd, SocketAddr)> {
    let sockfd = sockfd.as_fd();
    _accept4(sockfd, flags)
}

#[cfg(all(libc, not(any(target_os = "ios", target_os = "macos"))))]
fn _accept4(sockfd: BorrowedFd<'_>, flags: AcceptFlags) -> io::Result<(OwnedFd, SocketAddr)> {
    unsafe {
        let mut storage = MaybeUninit::<sockaddr_storage>::uninit();
        let mut len = size_of::<sockaddr_storage>() as socklen_t;
        let raw_fd = negone_err(libc::accept4(
            sockfd.as_raw_fd(),
            storage.as_mut_ptr() as *mut _,
            &mut len,
            flags.bits(),
        ))?;
        let owned_fd = OwnedFd::from_raw_fd(raw_fd);
        let storage = storage.assume_init();
        let addr = match i32::from(storage.ss_family) {
            libc::AF_INET => {
                assert!(len as usize >= size_of::<SocketAddrV4>());
                SocketAddr::V4((*(&storage as *const _ as *const SocketAddrV4)).clone())
            }
            libc::AF_INET6 => {
                assert!(len as usize >= size_of::<SocketAddrV6>());
                SocketAddr::V6((*(&storage as *const _ as *const SocketAddrV6)).clone())
            }
            libc::AF_LOCAL => {
                assert!(len as usize >= size_of::<SocketAddrUnix>());
                SocketAddr::Unix((*(&storage as *const _ as *const SocketAddrUnix)).clone())
            }
            _ => panic!(),
        };
        Ok((owned_fd, addr))
    }
}

#[cfg(linux_raw)]
#[inline]
fn _accept4(sockfd: BorrowedFd<'_>, flags: AcceptFlags) -> io::Result<(OwnedFd, SocketAddr)> {
    let (owned_fd, storage, len) = crate::linux_raw::accept4(sockfd, flags.bits())?;
    let addr = unsafe {
        match u32::from(storage.ss_family) {
            linux_raw_sys::general::AF_INET => {
                assert!(len as usize >= size_of::<SocketAddrV4>());
                SocketAddr::V4((*(&storage as *const _ as *const SocketAddrV4)).clone())
            }
            linux_raw_sys::general::AF_INET6 => {
                assert!(len as usize >= size_of::<SocketAddrV6>());
                SocketAddr::V6((*(&storage as *const _ as *const SocketAddrV6)).clone())
            }
            linux_raw_sys::general::AF_LOCAL => {
                assert!(len as usize >= size_of::<SocketAddrUnix>());
                SocketAddr::Unix((*(&storage as *const _ as *const SocketAddrUnix)).clone())
            }
            _ => panic!(),
        }
    };
    Ok((owned_fd, addr))
}

/// `shutdown(fd, how)`
#[inline]
pub fn shutdown<'f, Fd: AsFd<'f>>(sockfd: Fd, how: Shutdown) -> io::Result<()> {
    let sockfd = sockfd.as_fd();
    _shutdown(sockfd, how)
}

#[cfg(libc)]
fn _shutdown(sockfd: BorrowedFd<'_>, how: Shutdown) -> io::Result<()> {
    unsafe { zero_ok(libc::shutdown(sockfd.as_raw_fd(), how as c_int)) }
}

#[cfg(linux_raw)]
#[inline]
fn _shutdown(sockfd: BorrowedFd<'_>, how: Shutdown) -> io::Result<()> {
    crate::linux_raw::shutdown(sockfd, how as c_uint)
}

/// `getsockopt(fd, SOL_SOCKET, SO_TYPE)`
#[inline]
pub fn socket_type<'f, Fd: AsFd<'f>>(fd: Fd) -> io::Result<SocketType> {
    let fd = fd.as_fd();
    _socket_type(fd)
}

#[cfg(libc)]
fn _socket_type(fd: BorrowedFd<'_>) -> io::Result<SocketType> {
    let mut buffer = MaybeUninit::<SocketType>::uninit();
    let mut out_len = size_of::<SocketType>() as socklen_t;
    unsafe {
        zero_ok(libc::getsockopt(
            fd.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_TYPE,
            buffer.as_mut_ptr().cast::<libc::c_void>(),
            &mut out_len,
        ))?;
        assert_eq!(
            out_len as usize,
            size_of::<SocketType>(),
            "unexpected SocketType size"
        );
        Ok(buffer.assume_init())
    }
}

#[cfg(linux_raw)]
fn _socket_type(fd: BorrowedFd<'_>) -> io::Result<SocketType> {
    unsafe {
        let mut buffer = MaybeUninit::<SocketType>::uninit();
        let mut out_len = size_of::<SocketType>() as linux_raw_sys::general::socklen_t;
        let slice =
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, size_of::<SocketType>());
        crate::linux_raw::getsockopt(
            fd,
            linux_raw_sys::general::SOL_SOCKET as i32,
            linux_raw_sys::general::SO_TYPE as i32,
            slice,
            &mut out_len,
        )?;
        assert_eq!(
            out_len as usize,
            size_of::<SocketType>(),
            "unexpected SocketType size"
        );
        Ok(buffer.assume_init())
    }
}
