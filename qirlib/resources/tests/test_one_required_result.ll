; ModuleID = 'test_one_required_result'
source_filename = "test_one_required_result"

define void @main() #0 {
entry:
  ret void
}

attributes #0 = { "EntryPoint" "requiredQubits"="0" "requiredResults"="1" }
