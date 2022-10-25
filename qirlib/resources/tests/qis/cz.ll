; ModuleID = 'cz'
source_filename = "cz"

%Qubit = type opaque

define void @main() #0 {
entry:
  call void @__quantum__qis__cz__body(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*))
  ret void
}

declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="2" "requiredResults"="0" }
