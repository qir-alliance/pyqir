; ModuleID = 'attributes'
source_filename = "attributes"

declare "ret_attr"="ret value" i1 @foo(i64 "param0_attr"="param0 value" %0, double %1, i8* "param2_attr"="param2 value" %2) #0

attributes #0 = { "fn_attr"="fn value" }
