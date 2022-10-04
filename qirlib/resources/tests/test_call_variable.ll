; ModuleID = 'test_call_variable'
source_filename = "test_call_variable"

declare i64 @foo()

declare void @bar(i64)

define void @main() #0 {
entry:
  %0 = call i64 @foo()
  call void @bar(i64 %0)
  ret void
}

attributes #0 = { "EntryPoint" "requiredQubits"="0" "requiredResults"="0" }
