# typeshare-model

This crate is the base dependency for almost everything else. It includes especially the types and traits necessary for a single language implementation (that is, typeshare-swift could depend ONLY on typeshare-model). It can include utility functionality that might be necessary for a language implementation, but ideally not much in the way of major implementation stuff.
