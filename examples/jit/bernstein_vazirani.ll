; ModuleID = 'Bernstein-Vazirani'
source_filename = "./bernstein_vazirani.ll"

%Array = type opaque
%Qubit = type opaque
%Result = type opaque
%String = type opaque

@PauliX = internal constant i2 1
@PauliY = internal constant i2 -1
@PauliZ = internal constant i2 -2
@0 = internal constant [3 x i8] c", \00"
@1 = internal constant [2 x i8] c"[\00"
@2 = internal constant [3 x i8] c", \00"
@3 = internal constant [2 x i8] c"[\00"
@4 = internal constant [2 x i8] c"]\00"
@5 = internal constant [2 x i8] c"]\00"

define internal %Array* @QuantumApplication__Run__body() {
entry:
  %qubit0 = call %Qubit* @__quantum__rt__qubit_allocate()
  %qubit1 = call %Qubit* @__quantum__rt__qubit_allocate()
  %qubit2 = call %Qubit* @__quantum__rt__qubit_allocate()
  %qubit3 = call %Qubit* @__quantum__rt__qubit_allocate()
  %qubit4 = call %Qubit* @__quantum__rt__qubit_allocate()
  %qubit5 = call %Qubit* @__quantum__rt__qubit_allocate()
  %qubit6 = call %Qubit* @__quantum__rt__qubit_allocate()
  %qubit7 = call %Qubit* @__quantum__rt__qubit_allocate()
  %target0 = call %Qubit* @__quantum__rt__qubit_allocate()
  %results = call %Array* @__quantum__rt__array_create_1d(i32 8, i64 1)
  %output = call %Array* @__quantum__rt__array_create_1d(i32 8, i64 8)
  %output_0_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 0)
  %output_result_0 = bitcast i8* %output_0_raw to %Result**
  %zero_0 = call %Result* @__quantum__rt__result_get_zero()
  call void @__quantum__rt__result_update_reference_count(%Result* %zero_0, i32 1)
  store %Result* %zero_0, %Result** %output_result_0, align 8
  %output_1_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 1)
  %output_result_1 = bitcast i8* %output_1_raw to %Result**
  %zero_1 = call %Result* @__quantum__rt__result_get_zero()
  call void @__quantum__rt__result_update_reference_count(%Result* %zero_1, i32 1)
  store %Result* %zero_1, %Result** %output_result_1, align 8
  %output_2_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 2)
  %output_result_2 = bitcast i8* %output_2_raw to %Result**
  %zero_2 = call %Result* @__quantum__rt__result_get_zero()
  call void @__quantum__rt__result_update_reference_count(%Result* %zero_2, i32 1)
  store %Result* %zero_2, %Result** %output_result_2, align 8
  %output_3_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 3)
  %output_result_3 = bitcast i8* %output_3_raw to %Result**
  %zero_3 = call %Result* @__quantum__rt__result_get_zero()
  call void @__quantum__rt__result_update_reference_count(%Result* %zero_3, i32 1)
  store %Result* %zero_3, %Result** %output_result_3, align 8
  %output_4_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 4)
  %output_result_4 = bitcast i8* %output_4_raw to %Result**
  %zero_4 = call %Result* @__quantum__rt__result_get_zero()
  call void @__quantum__rt__result_update_reference_count(%Result* %zero_4, i32 1)
  store %Result* %zero_4, %Result** %output_result_4, align 8
  %output_5_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 5)
  %output_result_5 = bitcast i8* %output_5_raw to %Result**
  %zero_5 = call %Result* @__quantum__rt__result_get_zero()
  call void @__quantum__rt__result_update_reference_count(%Result* %zero_5, i32 1)
  store %Result* %zero_5, %Result** %output_result_5, align 8
  %output_6_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 6)
  %output_result_6 = bitcast i8* %output_6_raw to %Result**
  %zero_6 = call %Result* @__quantum__rt__result_get_zero()
  call void @__quantum__rt__result_update_reference_count(%Result* %zero_6, i32 1)
  store %Result* %zero_6, %Result** %output_result_6, align 8
  %output_7_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 7)
  %output_result_7 = bitcast i8* %output_7_raw to %Result**
  %zero_7 = call %Result* @__quantum__rt__result_get_zero()
  call void @__quantum__rt__result_update_reference_count(%Result* %zero_7, i32 1)
  store %Result* %zero_7, %Result** %output_result_7, align 8
  %results_result_tmp_0_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %results, i64 0)
  %results_result_tmp_result_0 = bitcast i8* %results_result_tmp_0_raw to %Array**
  store %Array* %output, %Array** %results_result_tmp_result_0, align 8
  call void @Microsoft__Quantum__Intrinsic__X__body(%Qubit* %target0)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit0)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit1)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit2)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit3)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit4)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit5)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit6)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit7)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %target0)
  %__controlQubits__ = call %Array* @__quantum__rt__array_create_1d(i32 8, i64 1)
  %__controlQubits__0_result_tmp_0_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %__controlQubits__, i64 0)
  %__controlQubits__0_result_tmp_result_0 = bitcast i8* %__controlQubits__0_result_tmp_0_raw to %Qubit**
  store %Qubit* %qubit2, %Qubit** %__controlQubits__0_result_tmp_result_0, align 8
  call void @Microsoft__Quantum__Intrinsic__X__ctl(%Array* %__controlQubits__, %Qubit* %target0)
  call void @__quantum__rt__array_update_reference_count(%Array* %__controlQubits__, i32 -1)
  %__controlQubits__1 = call %Array* @__quantum__rt__array_create_1d(i32 8, i64 1)
  %__controlQubits__0_result_tmp_0_raw2 = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %__controlQubits__1, i64 0)
  %__controlQubits__0_result_tmp_result_03 = bitcast i8* %__controlQubits__0_result_tmp_0_raw2 to %Qubit**
  store %Qubit* %qubit3, %Qubit** %__controlQubits__0_result_tmp_result_03, align 8
  call void @Microsoft__Quantum__Intrinsic__X__ctl(%Array* %__controlQubits__1, %Qubit* %target0)
  call void @__quantum__rt__array_update_reference_count(%Array* %__controlQubits__1, i32 -1)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit0)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit1)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit2)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit3)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit4)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit5)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit6)
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit7)
  call void @Microsoft__Quantum__Intrinsic__Reset__body(%Qubit* %target0)
  %measurement = call %Result* @Microsoft__Quantum__Intrinsic__M__body(%Qubit* %qubit0)
  %output0_0_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 0)
  %output0_result_0 = bitcast i8* %output0_0_raw to %Result**
  %existing_value = load %Result*, %Result** %output0_result_0, align 8
  call void @__quantum__rt__result_update_reference_count(%Result* %existing_value, i32 -1)
  call void @__quantum__rt__result_update_reference_count(%Result* %measurement, i32 1)
  store %Result* %measurement, %Result** %output0_result_0, align 8
  call void @Microsoft__Quantum__Intrinsic__Reset__body(%Qubit* %qubit0)
  %measurement4 = call %Result* @Microsoft__Quantum__Intrinsic__M__body(%Qubit* %qubit1)
  %output1_1_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 1)
  %output1_result_1 = bitcast i8* %output1_1_raw to %Result**
  %existing_value5 = load %Result*, %Result** %output1_result_1, align 8
  call void @__quantum__rt__result_update_reference_count(%Result* %existing_value5, i32 -1)
  call void @__quantum__rt__result_update_reference_count(%Result* %measurement4, i32 1)
  store %Result* %measurement4, %Result** %output1_result_1, align 8
  call void @Microsoft__Quantum__Intrinsic__Reset__body(%Qubit* %qubit1)
  %measurement6 = call %Result* @Microsoft__Quantum__Intrinsic__M__body(%Qubit* %qubit2)
  %output2_2_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 2)
  %output2_result_2 = bitcast i8* %output2_2_raw to %Result**
  %existing_value7 = load %Result*, %Result** %output2_result_2, align 8
  call void @__quantum__rt__result_update_reference_count(%Result* %existing_value7, i32 -1)
  call void @__quantum__rt__result_update_reference_count(%Result* %measurement6, i32 1)
  store %Result* %measurement6, %Result** %output2_result_2, align 8
  call void @Microsoft__Quantum__Intrinsic__Reset__body(%Qubit* %qubit2)
  %measurement8 = call %Result* @Microsoft__Quantum__Intrinsic__M__body(%Qubit* %qubit3)
  %output3_3_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 3)
  %output3_result_3 = bitcast i8* %output3_3_raw to %Result**
  %existing_value9 = load %Result*, %Result** %output3_result_3, align 8
  call void @__quantum__rt__result_update_reference_count(%Result* %existing_value9, i32 -1)
  call void @__quantum__rt__result_update_reference_count(%Result* %measurement8, i32 1)
  store %Result* %measurement8, %Result** %output3_result_3, align 8
  call void @Microsoft__Quantum__Intrinsic__Reset__body(%Qubit* %qubit3)
  %measurement10 = call %Result* @Microsoft__Quantum__Intrinsic__M__body(%Qubit* %qubit4)
  %output4_4_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 4)
  %output4_result_4 = bitcast i8* %output4_4_raw to %Result**
  %existing_value11 = load %Result*, %Result** %output4_result_4, align 8
  call void @__quantum__rt__result_update_reference_count(%Result* %existing_value11, i32 -1)
  call void @__quantum__rt__result_update_reference_count(%Result* %measurement10, i32 1)
  store %Result* %measurement10, %Result** %output4_result_4, align 8
  call void @Microsoft__Quantum__Intrinsic__Reset__body(%Qubit* %qubit4)
  %measurement12 = call %Result* @Microsoft__Quantum__Intrinsic__M__body(%Qubit* %qubit5)
  %output5_5_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 5)
  %output5_result_5 = bitcast i8* %output5_5_raw to %Result**
  %existing_value13 = load %Result*, %Result** %output5_result_5, align 8
  call void @__quantum__rt__result_update_reference_count(%Result* %existing_value13, i32 -1)
  call void @__quantum__rt__result_update_reference_count(%Result* %measurement12, i32 1)
  store %Result* %measurement12, %Result** %output5_result_5, align 8
  call void @Microsoft__Quantum__Intrinsic__Reset__body(%Qubit* %qubit5)
  %measurement14 = call %Result* @Microsoft__Quantum__Intrinsic__M__body(%Qubit* %qubit6)
  %output6_6_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 6)
  %output6_result_6 = bitcast i8* %output6_6_raw to %Result**
  %existing_value15 = load %Result*, %Result** %output6_result_6, align 8
  call void @__quantum__rt__result_update_reference_count(%Result* %existing_value15, i32 -1)
  call void @__quantum__rt__result_update_reference_count(%Result* %measurement14, i32 1)
  store %Result* %measurement14, %Result** %output6_result_6, align 8
  call void @Microsoft__Quantum__Intrinsic__Reset__body(%Qubit* %qubit6)
  %measurement16 = call %Result* @Microsoft__Quantum__Intrinsic__M__body(%Qubit* %qubit7)
  %output7_7_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 7)
  %output7_result_7 = bitcast i8* %output7_7_raw to %Result**
  %existing_value17 = load %Result*, %Result** %output7_result_7, align 8
  call void @__quantum__rt__result_update_reference_count(%Result* %existing_value17, i32 -1)
  call void @__quantum__rt__result_update_reference_count(%Result* %measurement16, i32 1)
  store %Result* %measurement16, %Result** %output7_result_7, align 8
  call void @Microsoft__Quantum__Intrinsic__Reset__body(%Qubit* %qubit7)
  call void @__quantum__rt__qubit_release(%Qubit* %qubit3)
  call void @__quantum__rt__qubit_release(%Qubit* %qubit0)
  call void @__quantum__rt__qubit_release(%Qubit* %qubit1)
  call void @__quantum__rt__qubit_release(%Qubit* %target0)
  call void @__quantum__rt__qubit_release(%Qubit* %qubit6)
  call void @__quantum__rt__qubit_release(%Qubit* %qubit7)
  call void @__quantum__rt__qubit_release(%Qubit* %qubit5)
  call void @__quantum__rt__qubit_release(%Qubit* %qubit2)
  call void @__quantum__rt__qubit_release(%Qubit* %qubit4)
  ret %Array* %results
}

declare %Qubit* @__quantum__rt__qubit_allocate()

declare void @__quantum__rt__qubit_release(%Qubit*)

declare %Array* @__quantum__rt__array_create_1d(i32, i64)

declare i8* @__quantum__rt__array_get_element_ptr_1d(%Array*, i64)

declare %Result* @__quantum__rt__result_get_zero()

declare void @__quantum__rt__result_update_reference_count(%Result*, i32)

declare void @__quantum__rt__array_update_alias_count(%Array*, i32)

declare i64 @__quantum__rt__array_get_size_1d(%Array*)

declare void @__quantum__rt__array_update_reference_count(%Array*, i32)

declare void @__quantum__qis__x__body(%Qubit*)

declare void @__quantum__qis__x__ctl(%Array*, %Qubit*)

declare void @__quantum__qis__h__body(%Qubit*)

define internal %Result* @Microsoft__Quantum__Intrinsic__M__body(%Qubit* %qubit) {
entry:
  %bases = call %Array* @__quantum__rt__array_create_1d(i32 1, i64 1)
  %0 = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %bases, i64 0)
  %1 = bitcast i8* %0 to i2*
  %2 = load i2, i2* @PauliZ, align 1
  store i2 %2, i2* %1, align 1
  call void @__quantum__rt__array_update_alias_count(%Array* %bases, i32 1)
  %qubits = call %Array* @__quantum__rt__array_create_1d(i32 8, i64 1)
  %3 = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %qubits, i64 0)
  %4 = bitcast i8* %3 to %Qubit**
  store %Qubit* %qubit, %Qubit** %4, align 8
  call void @__quantum__rt__array_update_alias_count(%Array* %qubits, i32 1)
  %5 = call %Result* @__quantum__qis__measure__body(%Array* %bases, %Array* %qubits)
  call void @__quantum__rt__array_update_alias_count(%Array* %bases, i32 -1)
  call void @__quantum__rt__array_update_alias_count(%Array* %qubits, i32 -1)
  call void @__quantum__rt__array_update_reference_count(%Array* %bases, i32 -1)
  call void @__quantum__rt__array_update_reference_count(%Array* %qubits, i32 -1)
  ret %Result* %5
}

define internal void @Microsoft__Quantum__Intrinsic__Reset__body(%Qubit* %qubit) {
entry:
  %0 = call %Result* @Microsoft__Quantum__Intrinsic__M__body(%Qubit* %qubit)
  %1 = call %Result* @__quantum__rt__result_get_one()
  %2 = call i1 @__quantum__rt__result_equal(%Result* %0, %Result* %1)
  call void @__quantum__rt__result_update_reference_count(%Result* %0, i32 -1)
  br i1 %2, label %then0__1, label %continue__1

then0__1:                                         ; preds = %entry
  call void @__quantum__qis__x__body(%Qubit* %qubit)
  br label %continue__1

continue__1:                                      ; preds = %then0__1, %entry
  ret void
}

define internal void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit) {
entry:
  call void @__quantum__qis__h__body(%Qubit* %qubit)
  ret void
}

declare %Result* @__quantum__qis__measure__body(%Array*, %Array*)

declare %Result* @__quantum__rt__result_get_one()

declare i1 @__quantum__rt__result_equal(%Result*, %Result*)

define internal void @Microsoft__Quantum__Intrinsic__X__body(%Qubit* %qubit) {
entry:
  call void @__quantum__qis__x__body(%Qubit* %qubit)
  ret void
}

define internal void @Microsoft__Quantum__Intrinsic__X__ctl(%Array* %__controlQubits__, %Qubit* %qubit) {
entry:
  call void @__quantum__rt__array_update_alias_count(%Array* %__controlQubits__, i32 1)
  call void @__quantum__qis__x__ctl(%Array* %__controlQubits__, %Qubit* %qubit)
  call void @__quantum__rt__array_update_alias_count(%Array* %__controlQubits__, i32 -1)
  ret void
}

define { i64, i8* }* @QuantumApplication__Run__Interop() #0 {
entry:
  %0 = call %Array* @QuantumApplication__Run__body()
  %1 = call i64 @__quantum__rt__array_get_size_1d(%Array* %0)
  %2 = mul i64 %1, 8
  %3 = call i8* @__quantum__rt__memory_allocate(i64 %2)
  %4 = ptrtoint i8* %3 to i64
  %5 = sub i64 %1, 1
  br label %header__1

header__1:                                        ; preds = %exiting__1, %entry
  %6 = phi i64 [ 0, %entry ], [ %19, %exiting__1 ]
  %7 = icmp sle i64 %6, %5
  br i1 %7, label %body__1, label %exit__1

body__1:                                          ; preds = %header__1
  %8 = mul i64 %6, 8
  %9 = add i64 %4, %8
  %10 = inttoptr i64 %9 to { i64, i8* }**
  %11 = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %0, i64 %6)
  %12 = bitcast i8* %11 to %Array**
  %13 = load %Array*, %Array** %12, align 8
  %14 = call i64 @__quantum__rt__array_get_size_1d(%Array* %13)
  %15 = mul i64 %14, 1
  %16 = call i8* @__quantum__rt__memory_allocate(i64 %15)
  %17 = ptrtoint i8* %16 to i64
  %18 = sub i64 %14, 1
  br label %header__2

exiting__1:                                       ; preds = %exit__2
  %19 = add i64 %6, 1
  br label %header__1

exit__1:                                          ; preds = %header__1
  %20 = call i8* @__quantum__rt__memory_allocate(i64 ptrtoint ({ i64, i8* }* getelementptr ({ i64, i8* }, { i64, i8* }* null, i32 1) to i64))
  %21 = bitcast i8* %20 to { i64, i8* }*
  %22 = getelementptr { i64, i8* }, { i64, i8* }* %21, i64 0, i32 0
  store i64 %1, i64* %22, align 4
  %23 = getelementptr { i64, i8* }, { i64, i8* }* %21, i64 0, i32 1
  store i8* %3, i8** %23, align 8
  %24 = sub i64 %1, 1
  br label %header__3

header__2:                                        ; preds = %exiting__2, %body__1
  %25 = phi i64 [ 0, %body__1 ], [ %36, %exiting__2 ]
  %26 = icmp sle i64 %25, %18
  br i1 %26, label %body__2, label %exit__2

body__2:                                          ; preds = %header__2
  %27 = mul i64 %25, 1
  %28 = add i64 %17, %27
  %29 = inttoptr i64 %28 to i8*
  %30 = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %13, i64 %25)
  %31 = bitcast i8* %30 to %Result**
  %32 = load %Result*, %Result** %31, align 8
  %33 = call %Result* @__quantum__rt__result_get_zero()
  %34 = call i1 @__quantum__rt__result_equal(%Result* %32, %Result* %33)
  %35 = select i1 %34, i8 0, i8 -1
  store i8 %35, i8* %29, align 1
  br label %exiting__2

exiting__2:                                       ; preds = %body__2
  %36 = add i64 %25, 1
  br label %header__2

exit__2:                                          ; preds = %header__2
  %37 = call i8* @__quantum__rt__memory_allocate(i64 ptrtoint ({ i64, i8* }* getelementptr ({ i64, i8* }, { i64, i8* }* null, i32 1) to i64))
  %38 = bitcast i8* %37 to { i64, i8* }*
  %39 = getelementptr { i64, i8* }, { i64, i8* }* %38, i64 0, i32 0
  store i64 %14, i64* %39, align 4
  %40 = getelementptr { i64, i8* }, { i64, i8* }* %38, i64 0, i32 1
  store i8* %16, i8** %40, align 8
  store { i64, i8* }* %38, { i64, i8* }** %10, align 8
  br label %exiting__1

header__3:                                        ; preds = %exiting__3, %exit__1
  %41 = phi i64 [ 0, %exit__1 ], [ %48, %exiting__3 ]
  %42 = icmp sle i64 %41, %24
  br i1 %42, label %body__3, label %exit__3

body__3:                                          ; preds = %header__3
  %43 = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %0, i64 %41)
  %44 = bitcast i8* %43 to %Array**
  %45 = load %Array*, %Array** %44, align 8
  %46 = call i64 @__quantum__rt__array_get_size_1d(%Array* %45)
  %47 = sub i64 %46, 1
  br label %header__4

exiting__3:                                       ; preds = %exit__4
  %48 = add i64 %41, 1
  br label %header__3

exit__3:                                          ; preds = %header__3
  call void @__quantum__rt__array_update_reference_count(%Array* %0, i32 -1)
  ret { i64, i8* }* %21

header__4:                                        ; preds = %exiting__4, %body__3
  %49 = phi i64 [ 0, %body__3 ], [ %54, %exiting__4 ]
  %50 = icmp sle i64 %49, %47
  br i1 %50, label %body__4, label %exit__4

body__4:                                          ; preds = %header__4
  %51 = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %45, i64 %49)
  %52 = bitcast i8* %51 to %Result**
  %53 = load %Result*, %Result** %52, align 8
  call void @__quantum__rt__result_update_reference_count(%Result* %53, i32 -1)
  br label %exiting__4

exiting__4:                                       ; preds = %body__4
  %54 = add i64 %49, 1
  br label %header__4

exit__4:                                          ; preds = %header__4
  call void @__quantum__rt__array_update_reference_count(%Array* %45, i32 -1)
  br label %exiting__3
}

declare i8* @__quantum__rt__memory_allocate(i64)

define void @QuantumApplication__Run() #1 {
entry:
  %0 = call %Array* @QuantumApplication__Run__body()
  %1 = call %String* @__quantum__rt__string_create(i8* getelementptr inbounds ([3 x i8], [3 x i8]* @0, i32 0, i32 0))
  %2 = call %String* @__quantum__rt__string_create(i8* getelementptr inbounds ([2 x i8], [2 x i8]* @1, i32 0, i32 0))
  call void @__quantum__rt__string_update_reference_count(%String* %2, i32 1)
  %3 = call i64 @__quantum__rt__array_get_size_1d(%Array* %0)
  %4 = sub i64 %3, 1
  br label %header__1

header__1:                                        ; preds = %exiting__1, %entry
  %5 = phi %String* [ %2, %entry ], [ %36, %exiting__1 ]
  %6 = phi i64 [ 0, %entry ], [ %18, %exiting__1 ]
  %7 = icmp sle i64 %6, %4
  br i1 %7, label %body__1, label %exit__1

body__1:                                          ; preds = %header__1
  %8 = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %0, i64 %6)
  %9 = bitcast i8* %8 to %Array**
  %10 = load %Array*, %Array** %9, align 8
  %11 = icmp ne %String* %5, %2
  br i1 %11, label %condTrue__1, label %condContinue__1

condTrue__1:                                      ; preds = %body__1
  %12 = call %String* @__quantum__rt__string_concatenate(%String* %5, %String* %1)
  call void @__quantum__rt__string_update_reference_count(%String* %5, i32 -1)
  br label %condContinue__1

condContinue__1:                                  ; preds = %condTrue__1, %body__1
  %13 = phi %String* [ %12, %condTrue__1 ], [ %5, %body__1 ]
  %14 = call %String* @__quantum__rt__string_create(i8* getelementptr inbounds ([3 x i8], [3 x i8]* @2, i32 0, i32 0))
  %15 = call %String* @__quantum__rt__string_create(i8* getelementptr inbounds ([2 x i8], [2 x i8]* @3, i32 0, i32 0))
  call void @__quantum__rt__string_update_reference_count(%String* %15, i32 1)
  %16 = call i64 @__quantum__rt__array_get_size_1d(%Array* %10)
  %17 = sub i64 %16, 1
  br label %header__2

exiting__1:                                       ; preds = %exit__2
  %18 = add i64 %6, 1
  br label %header__1

exit__1:                                          ; preds = %header__1
  %19 = call %String* @__quantum__rt__string_create(i8* getelementptr inbounds ([2 x i8], [2 x i8]* @5, i32 0, i32 0))
  %20 = call %String* @__quantum__rt__string_concatenate(%String* %5, %String* %19)
  call void @__quantum__rt__string_update_reference_count(%String* %5, i32 -1)
  call void @__quantum__rt__string_update_reference_count(%String* %19, i32 -1)
  call void @__quantum__rt__string_update_reference_count(%String* %1, i32 -1)
  call void @__quantum__rt__string_update_reference_count(%String* %2, i32 -1)
  call void @__quantum__rt__message(%String* %20)
  %21 = sub i64 %3, 1
  br label %header__3

header__2:                                        ; preds = %exiting__2, %condContinue__1
  %22 = phi %String* [ %15, %condContinue__1 ], [ %32, %exiting__2 ]
  %23 = phi i64 [ 0, %condContinue__1 ], [ %33, %exiting__2 ]
  %24 = icmp sle i64 %23, %17
  br i1 %24, label %body__2, label %exit__2

body__2:                                          ; preds = %header__2
  %25 = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %10, i64 %23)
  %26 = bitcast i8* %25 to %Result**
  %27 = load %Result*, %Result** %26, align 8
  %28 = icmp ne %String* %22, %15
  br i1 %28, label %condTrue__2, label %condContinue__2

condTrue__2:                                      ; preds = %body__2
  %29 = call %String* @__quantum__rt__string_concatenate(%String* %22, %String* %14)
  call void @__quantum__rt__string_update_reference_count(%String* %22, i32 -1)
  br label %condContinue__2

condContinue__2:                                  ; preds = %condTrue__2, %body__2
  %30 = phi %String* [ %29, %condTrue__2 ], [ %22, %body__2 ]
  %31 = call %String* @__quantum__rt__result_to_string(%Result* %27)
  %32 = call %String* @__quantum__rt__string_concatenate(%String* %30, %String* %31)
  call void @__quantum__rt__string_update_reference_count(%String* %30, i32 -1)
  call void @__quantum__rt__string_update_reference_count(%String* %31, i32 -1)
  br label %exiting__2

exiting__2:                                       ; preds = %condContinue__2
  %33 = add i64 %23, 1
  br label %header__2

exit__2:                                          ; preds = %header__2
  %34 = call %String* @__quantum__rt__string_create(i8* getelementptr inbounds ([2 x i8], [2 x i8]* @4, i32 0, i32 0))
  %35 = call %String* @__quantum__rt__string_concatenate(%String* %22, %String* %34)
  call void @__quantum__rt__string_update_reference_count(%String* %22, i32 -1)
  call void @__quantum__rt__string_update_reference_count(%String* %34, i32 -1)
  call void @__quantum__rt__string_update_reference_count(%String* %14, i32 -1)
  call void @__quantum__rt__string_update_reference_count(%String* %15, i32 -1)
  %36 = call %String* @__quantum__rt__string_concatenate(%String* %13, %String* %35)
  call void @__quantum__rt__string_update_reference_count(%String* %13, i32 -1)
  call void @__quantum__rt__string_update_reference_count(%String* %35, i32 -1)
  br label %exiting__1

header__3:                                        ; preds = %exiting__3, %exit__1
  %37 = phi i64 [ 0, %exit__1 ], [ %44, %exiting__3 ]
  %38 = icmp sle i64 %37, %21
  br i1 %38, label %body__3, label %exit__3

body__3:                                          ; preds = %header__3
  %39 = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %0, i64 %37)
  %40 = bitcast i8* %39 to %Array**
  %41 = load %Array*, %Array** %40, align 8
  %42 = call i64 @__quantum__rt__array_get_size_1d(%Array* %41)
  %43 = sub i64 %42, 1
  br label %header__4

exiting__3:                                       ; preds = %exit__4
  %44 = add i64 %37, 1
  br label %header__3

exit__3:                                          ; preds = %header__3
  call void @__quantum__rt__array_update_reference_count(%Array* %0, i32 -1)
  call void @__quantum__rt__string_update_reference_count(%String* %20, i32 -1)
  ret void

header__4:                                        ; preds = %exiting__4, %body__3
  %45 = phi i64 [ 0, %body__3 ], [ %50, %exiting__4 ]
  %46 = icmp sle i64 %45, %43
  br i1 %46, label %body__4, label %exit__4

body__4:                                          ; preds = %header__4
  %47 = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %41, i64 %45)
  %48 = bitcast i8* %47 to %Result**
  %49 = load %Result*, %Result** %48, align 8
  call void @__quantum__rt__result_update_reference_count(%Result* %49, i32 -1)
  br label %exiting__4

exiting__4:                                       ; preds = %body__4
  %50 = add i64 %45, 1
  br label %header__4

exit__4:                                          ; preds = %header__4
  call void @__quantum__rt__array_update_reference_count(%Array* %41, i32 -1)
  br label %exiting__3
}

declare void @__quantum__rt__message(%String*)

declare %String* @__quantum__rt__string_create(i8*)

declare void @__quantum__rt__string_update_reference_count(%String*, i32)

declare %String* @__quantum__rt__string_concatenate(%String*, %String*)

declare %String* @__quantum__rt__result_to_string(%Result*)

attributes #0 = { "InteropFriendly" }
attributes #1 = { "EntryPoint" }