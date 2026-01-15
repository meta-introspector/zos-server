; ModuleID = 'helloworld.9f4e2c8587acf621-cgu.0'
source_filename = "helloworld.9f4e2c8587acf621-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@vtable.0 = private unnamed_addr constant <{ [24 x i8], ptr, ptr, ptr }> <{ [24 x i8] c"\00\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", ptr @_RNSNvYNCINvNtCs85GBm0N6DSS_3std2rt10lang_startuE0INtNtNtCs8vqiuRab9r0_4core3ops8function6FnOnceuE9call_once6vtableCsdFYGeFGU6w7_10helloworld, ptr @_RNCINvNtCs85GBm0N6DSS_3std2rt10lang_startuE0CsdFYGeFGU6w7_10helloworld, ptr @_RNCINvNtCs85GBm0N6DSS_3std2rt10lang_startuE0CsdFYGeFGU6w7_10helloworld }>, align 8
@alloc_3213114faf700a46436312d7d5d956d1 = private unnamed_addr constant [14 x i8] c"Hello, world!\0A", align 1

; std::rt::lang_start::<()>
; Function Attrs: nonlazybind uwtable
define hidden i64 @_RINvNtCs85GBm0N6DSS_3std2rt10lang_startuECsdFYGeFGU6w7_10helloworld(ptr %main, i64 %argc, ptr %argv, i8 %sigpipe) unnamed_addr #0 {
start:
  %_7 = alloca [8 x i8], align 8
  store ptr %main, ptr %_7, align 8
; call std::rt::lang_start_internal
  %_0 = call i64 @_RNvNtCs85GBm0N6DSS_3std2rt19lang_start_internal(ptr align 1 %_7, ptr align 8 @vtable.0, i64 %argc, ptr %argv, i8 %sigpipe)
  ret i64 %_0
}

; std::sys::backtrace::__rust_begin_short_backtrace::<fn(), ()>
; Function Attrs: noinline nonlazybind uwtable
define internal void @_RINvNtNtCs85GBm0N6DSS_3std3sys9backtrace28___rust_begin_short_backtraceFEuuECsdFYGeFGU6w7_10helloworld(ptr %f) unnamed_addr #1 {
start:
; call <fn() as core::ops::function::FnOnce<()>>::call_once
  call void @_RNvYFEuINtNtNtCs8vqiuRab9r0_4core3ops8function6FnOnceuE9call_onceCsdFYGeFGU6w7_10helloworld(ptr %f) #5
  call void asm sideeffect "", "~{memory}"(), !srcloc !4
  ret void
}

; std::rt::lang_start::<()>::{closure#0}
; Function Attrs: inlinehint nonlazybind uwtable
define internal i32 @_RNCINvNtCs85GBm0N6DSS_3std2rt10lang_startuE0CsdFYGeFGU6w7_10helloworld(ptr align 8 %_1) unnamed_addr #2 {
start:
  %_4 = load ptr, ptr %_1, align 8
; call std::sys::backtrace::__rust_begin_short_backtrace::<fn(), ()>
  call void @_RINvNtNtCs85GBm0N6DSS_3std3sys9backtrace28___rust_begin_short_backtraceFEuuECsdFYGeFGU6w7_10helloworld(ptr %_4) #6
; call <() as std::process::Termination>::report
  %self = call i8 @_RNvXsZ_NtCs85GBm0N6DSS_3std7processuNtB5_11Termination6reportCsdFYGeFGU6w7_10helloworld() #5
  %_0 = zext i8 %self to i32
  ret i32 %_0
}

; <std::rt::lang_start<()>::{closure#0} as core::ops::function::FnOnce<()>>::call_once::{shim:vtable#0}
; Function Attrs: inlinehint nonlazybind uwtable
define internal i32 @_RNSNvYNCINvNtCs85GBm0N6DSS_3std2rt10lang_startuE0INtNtNtCs8vqiuRab9r0_4core3ops8function6FnOnceuE9call_once6vtableCsdFYGeFGU6w7_10helloworld(ptr %_1) unnamed_addr #2 {
start:
  %_2 = alloca [0 x i8], align 1
  %0 = load ptr, ptr %_1, align 8
; call <std::rt::lang_start<()>::{closure#0} as core::ops::function::FnOnce<()>>::call_once
  %_0 = call i32 @_RNvYNCINvNtCs85GBm0N6DSS_3std2rt10lang_startuE0INtNtNtCs8vqiuRab9r0_4core3ops8function6FnOnceuE9call_onceCsdFYGeFGU6w7_10helloworld(ptr %0) #5
  ret i32 %_0
}

; helloworld::main
; Function Attrs: nonlazybind uwtable
define hidden void @_RNvCsdFYGeFGU6w7_10helloworld4main() unnamed_addr #0 {
start:
; call <core::fmt::Arguments>::from_str
  %0 = call { ptr, ptr } @_RNvMs4_NtCs8vqiuRab9r0_4core3fmtNtB5_9Arguments8from_strCsdFYGeFGU6w7_10helloworld(ptr align 1 @alloc_3213114faf700a46436312d7d5d956d1, i64 14) #5
  %_2.0 = extractvalue { ptr, ptr } %0, 0
  %_2.1 = extractvalue { ptr, ptr } %0, 1
; call std::io::stdio::_print
  call void @_RNvNtNtCs85GBm0N6DSS_3std2io5stdio6__print(ptr %_2.0, ptr %_2.1)
  ret void
}

; <core::fmt::Arguments>::from_str
; Function Attrs: inlinehint nonlazybind uwtable
define internal { ptr, ptr } @_RNvMs4_NtCs8vqiuRab9r0_4core3fmtNtB5_9Arguments8from_strCsdFYGeFGU6w7_10helloworld(ptr align 1 %s.0, i64 %s.1) unnamed_addr #2 {
start:
  %_6 = shl i64 %s.1, 1
  %_5 = or i64 %_6, 1
  %_4 = inttoptr i64 %_5 to ptr
  %0 = insertvalue { ptr, ptr } poison, ptr %s.0, 0
  %1 = insertvalue { ptr, ptr } %0, ptr %_4, 1
  ret { ptr, ptr } %1
}

; <() as std::process::Termination>::report
; Function Attrs: inlinehint nonlazybind uwtable
define internal i8 @_RNvXsZ_NtCs85GBm0N6DSS_3std7processuNtB5_11Termination6reportCsdFYGeFGU6w7_10helloworld() unnamed_addr #2 {
start:
  ret i8 0
}

; <fn() as core::ops::function::FnOnce<()>>::call_once
; Function Attrs: inlinehint nonlazybind uwtable
define internal void @_RNvYFEuINtNtNtCs8vqiuRab9r0_4core3ops8function6FnOnceuE9call_onceCsdFYGeFGU6w7_10helloworld(ptr %_1) unnamed_addr #2 {
start:
  %_2 = alloca [0 x i8], align 1
  call void %_1()
  ret void
}

; <std::rt::lang_start<()>::{closure#0} as core::ops::function::FnOnce<()>>::call_once
; Function Attrs: inlinehint nonlazybind uwtable
define internal i32 @_RNvYNCINvNtCs85GBm0N6DSS_3std2rt10lang_startuE0INtNtNtCs8vqiuRab9r0_4core3ops8function6FnOnceuE9call_onceCsdFYGeFGU6w7_10helloworld(ptr %0) unnamed_addr #2 personality ptr @rust_eh_personality {
start:
  %1 = alloca [16 x i8], align 8
  %_2 = alloca [0 x i8], align 1
  %_1 = alloca [8 x i8], align 8
  store ptr %0, ptr %_1, align 8
; invoke std::rt::lang_start::<()>::{closure#0}
  %_0 = invoke i32 @_RNCINvNtCs85GBm0N6DSS_3std2rt10lang_startuE0CsdFYGeFGU6w7_10helloworld(ptr align 8 %_1)
          to label %bb1 unwind label %cleanup

bb3:                                              ; preds = %cleanup
  %2 = load ptr, ptr %1, align 8
  %3 = getelementptr inbounds i8, ptr %1, i64 8
  %4 = load i32, ptr %3, align 8
  %5 = insertvalue { ptr, i32 } poison, ptr %2, 0
  %6 = insertvalue { ptr, i32 } %5, i32 %4, 1
  resume { ptr, i32 } %6

cleanup:                                          ; preds = %start
  %7 = landingpad { ptr, i32 }
          cleanup
  %8 = extractvalue { ptr, i32 } %7, 0
  %9 = extractvalue { ptr, i32 } %7, 1
  store ptr %8, ptr %1, align 8
  %10 = getelementptr inbounds i8, ptr %1, i64 8
  store i32 %9, ptr %10, align 8
  br label %bb3

bb1:                                              ; preds = %start
  ret i32 %_0
}

; std::rt::lang_start_internal
; Function Attrs: nonlazybind uwtable
declare i64 @_RNvNtCs85GBm0N6DSS_3std2rt19lang_start_internal(ptr align 1, ptr align 8, i64, ptr, i8) unnamed_addr #0

; std::io::stdio::_print
; Function Attrs: nonlazybind uwtable
declare void @_RNvNtNtCs85GBm0N6DSS_3std2io5stdio6__print(ptr, ptr) unnamed_addr #0

; Function Attrs: nounwind nonlazybind uwtable
declare i32 @rust_eh_personality(i32, i32, i64, ptr, ptr) unnamed_addr #3

; Function Attrs: nonlazybind
define i32 @main(i32 %0, ptr %1) unnamed_addr #4 {
top:
  %2 = sext i32 %0 to i64
; call std::rt::lang_start::<()>
  %3 = call i64 @_RINvNtCs85GBm0N6DSS_3std2rt10lang_startuECsdFYGeFGU6w7_10helloworld(ptr @_RNvCsdFYGeFGU6w7_10helloworld4main, i64 %2, ptr %1, i8 0)
  %4 = trunc i64 %3 to i32
  ret i32 %4
}

attributes #0 = { nonlazybind uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #1 = { noinline nonlazybind uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #2 = { inlinehint nonlazybind uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #3 = { nounwind nonlazybind uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #4 = { nonlazybind "target-cpu"="x86-64" }
attributes #5 = { inlinehint }
attributes #6 = { noinline }

!llvm.module.flags = !{!0, !1, !2}
!llvm.ident = !{!3}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{i32 7, !"PIE Level", i32 2}
!2 = !{i32 2, !"RtLibUseGOT", i32 1}
!3 = !{!"rustc version 1.94.0-nightly (31cd367b9 2026-01-08)"}
!4 = !{i64 5722644491293844}
