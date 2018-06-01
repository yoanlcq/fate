bitflags! {
    /// This bitfield is used as a set for each thread and task.
    /// A thread is allowed to consume a task, if and only if their
    /// sets intersect.
    ///
    /// `ANY` and `ONLY_MAIN_THREAD` are the only two "standard" flags.
    /// You may add or remove others depending on your use cases.
    pub struct ThreadMask: u32 {
        /// This flag exists as the single catch-all for most
        /// tasks and some threads.
        ///
        /// When a thread has no raised flag, it's not allowed
        /// to consume any task at all. Likewise, when a task has no
        /// raised flag, no thread is allowed to consume it.
        ///
        /// However, some tasks are too general-purpose to fit in
        /// other categories, and some threads have no specific role
        /// either.
        /// In order for these tasks and threads to be useful at all,
        /// they should at a minimum raise this flag.
        const ANY = 0b00000001;
        /// The only thread that can (and must) have the `ONLY_MAIN_THREAD`
        /// flag set is, obviously, the main thread.
        /// Tasks that set this bit will therefore always be executed
        /// in the main thread, which is required by some APIs such as
        /// OpenGL and window event pumps.
        const ONLY_MAIN_THREAD = 0b000000010;

        // NOTE: "engine" flags start from 8th bit right now.

        /// Flag for persistent storage I/O.
        const FILE_IO = 0b100000000;
        /// Flag for network I/O.
        const NETWORK_IO = 0b1000000000;
        /// Union of all I/O flags.
        const IO = Self::FILE_IO.bits | Self::NETWORK_IO.bits;

        // NOTE: "game" flags start from 16th bit right now.
        // - GFX would fit in ONLY_MAIN_THREAD as long as we're using OpenGL;
        // - GAME_LOGIC would fit either in PHYSICS or in ANY.

        /// Flag for audio I/O and DSP.
        const AUDIO = 0b10000000000000000;
        /// Flag for physics calculations.
        const PHYSICS = 0b100000000000000000;
        /// Flag for AI and pathfinding.
        const AI = 0b1000000000000000000;
        /// Flag for on-CPU skeletal animation and blending.
        const ANIM = 0b10000000000000000000;
    }
}
// The default value for `ThreadMask` is `ONLY_MAIN_THREAD`, which
// is always a safe default.
impl Default for ThreadMask {
    fn default() -> Self {
        Self::ONLY_MAIN_THREAD
    }
}