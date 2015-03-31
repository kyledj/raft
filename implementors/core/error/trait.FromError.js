(function() {var implementors = {};
implementors['capnp'] = ["<a class='stability Stable' title='Stable'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='struct' href='http://doc.rust-lang.org/nightly/std/io/error/struct.Error.html' title='std::io::error::Error'>Error</a>&gt; for <a class='enum' href='capnp/enum.Error.html' title='capnp::Error'>Error</a>","<a class='stability Stable' title='Stable'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='struct' href='capnp/struct.NotInSchema.html' title='capnp::NotInSchema'>NotInSchema</a>&gt; for <a class='enum' href='capnp/enum.Error.html' title='capnp::Error'>Error</a>","<a class='stability Stable' title='Stable'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='enum' href='http://doc.rust-lang.org/nightly/core/str/enum.Utf8Error.html' title='core::str::Utf8Error'>Utf8Error</a>&gt; for <a class='enum' href='capnp/enum.Error.html' title='capnp::Error'>Error</a>",];implementors['mio'] = ["<a class='stability Stable' title='Stable'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='struct' href='http://doc.rust-lang.org/nightly/std/ffi/c_str/struct.NulError.html' title='std::ffi::c_str::NulError'>NulError</a>&gt; for <a class='struct' href='std/io/error/struct.Error.html' title='std::io::error::Error'>Error</a>","<a class='stability Stable' title='Stable'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='struct' href='http://doc.rust-lang.org/nightly/std/ffi/c_str/struct.NulError.html' title='std::ffi::c_str::NulError'>NulError</a>&gt; for <a class='struct' href='http://doc.rust-lang.org/nightly/std/old_io/struct.IoError.html' title='std::old_io::IoError'>IoError</a>","<a class='stability Stable' title='Stable'></a>impl&lt;W&gt; <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='struct' href='http://doc.rust-lang.org/nightly/std/io/buffered/struct.IntoInnerError.html' title='std::io::buffered::IntoInnerError'>IntoInnerError</a>&lt;W&gt;&gt; for <a class='struct' href='std/io/error/struct.Error.html' title='std::io::error::Error'>Error</a>","<a class='stability Stable' title='Stable'></a>impl&lt;T&gt; <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='struct' href='http://doc.rust-lang.org/nightly/std/sync/poison/struct.PoisonError.html' title='std::sync::poison::PoisonError'>PoisonError</a>&lt;T&gt;&gt; for <a class='enum' href='http://doc.rust-lang.org/nightly/std/sync/poison/enum.TryLockError.html' title='std::sync::poison::TryLockError'>TryLockError</a>&lt;T&gt;","<a class='stability Stable' title='Stable'></a>impl&lt;'a, E&gt; <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;E&gt; for <a class='struct' href='http://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html' title='alloc::boxed::Box'>Box</a>&lt;<a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.Error.html' title='core::error::Error'>Error</a> + 'a&gt;",];implementors['rustc_serialize'] = ["<a class='stability Stable' title='Stable'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='struct' href='http://doc.rust-lang.org/nightly/core/fmt/struct.Error.html' title='core::fmt::Error'>Error</a>&gt; for <a class='enum' href='rustc_serialize/json/enum.EncoderError.html' title='rustc_serialize::json::EncoderError'>EncoderError</a>",];implementors['raft'] = ["<a class='stability Stable' title='Stable'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='struct' href='http://doc.rust-lang.org/nightly/std/io/error/struct.Error.html' title='std::io::error::Error'>Error</a>&gt; for <a class='enum' href='raft/enum.Error.html' title='raft::Error'>Error</a>","<a class='stability Stable' title='Stable'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='enum' href='capnp/enum.Error.html' title='capnp::Error'>Error</a>&gt; for <a class='enum' href='raft/enum.Error.html' title='raft::Error'>Error</a>","<a class='stability Stable' title='Stable'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='struct' href='capnp/struct.NotInSchema.html' title='capnp::NotInSchema'>NotInSchema</a>&gt; for <a class='enum' href='raft/enum.Error.html' title='raft::Error'>Error</a>",];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
