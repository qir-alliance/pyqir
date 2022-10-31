# PyQIR

##  Safety

To store Inkwell objects in Python classes, we transmute the `'ctx` lifetime to static.
You need to be careful when using Inkwell types with unsafely extended lifetimes.
Follow these rules:

1. When storing in a data type, always include a `Py<Context>` field containing the context originally referred to by `'ctx`.
2. Before calling Inkwell methods that use `'ctx`, call `context::require_same` to assert that all contexts being used are the same.
