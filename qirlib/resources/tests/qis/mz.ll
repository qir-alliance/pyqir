; ModuleID = 'mz'
source_filename = "mz"

%Qubit = type opaque
%Result = type opaque

define void @main() #0 {
entry:
  call void @__quantum__qis__mz__body(%Qubit* null, %Result* null)
  ret void
}

declare void @__quantum__qis__mz__body(%Qubit*, %Result*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" "requiredResults"="1" }
