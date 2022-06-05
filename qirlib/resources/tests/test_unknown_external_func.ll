; ModuleID = 'test_unknown_external_func'
source_filename = "test_unknown_external_func"

%String = type opaque

declare %String* @__quantum__rt__bool_to_string(i1)

define void @main() #1 {
entry:
  call %String* @__quantum__rt__bool_to_string(i1 1)
  ret void
}

attributes #1 = { "EntryPoint" }
