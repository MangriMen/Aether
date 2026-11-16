use process_wrap::tokio::CommandWrap;

/// Makes `command` start a process tree that can be stopped as a whole: the process is the leader
/// of a job object on Windows, or of a process group on Unix. Stopping the child then stops
/// every process it started too.
///
/// A process that leaves on purpose (`setsid` on Unix, a breakaway from the job on Windows) still
/// escapes. On Windows the job object replaces the creation flags set on the command: any the
/// process needs must be given through `CreationFlags`, wrapped before this.
pub fn wrap_tree(command: &mut CommandWrap) {
    #[cfg(windows)]
    command.wrap(process_wrap::tokio::JobObject);
    #[cfg(unix)]
    command.wrap(process_wrap::tokio::ProcessGroup::leader());
}
