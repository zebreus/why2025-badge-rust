// WARNING: This file is LLM-generated based on the list of builtin functions in `symbols.yml`

#ifndef GCC_BUILTINS_H
#define GCC_BUILTINS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Double-precision floating-point arithmetic */
extern double __adddf3(double a, double b);
extern double __subdf3(double a, double b);
extern double __muldf3(double a, double b);
extern double __divdf3(double a, double b);

/* Double-precision floating-point comparisons (return <0, 0, >0) */
extern int __eqdf2(double a, double b);
extern int __gedf2(double a, double b);
extern int __gtdf2(double a, double b);
extern int __ledf2(double a, double b);
extern int __ltdf2(double a, double b);

/* Conversion between float/double and integer types */
extern double __extendsfdf2(float a);
extern float  __truncdfsf2(double a);

extern float  __extendhfsf2(_Float16 a);
extern _Float16 __truncsfhf2(float a);

extern int32_t  __fixdfsi(double a);
extern int64_t  __fixdfdi(double a);
extern uint32_t __fixunsdfsi(double a);

extern float    __floatdisf(int64_t a);
extern double   __floatsidf(int32_t a);
extern double   __floatunsidf(uint32_t a);

/* Double-precision division for 64-bit integers */
extern int64_t  __divdi3(int64_t a, int64_t b);
extern uint64_t __udivdi3(uint64_t a, uint64_t b);
extern uint64_t __umoddi3(uint64_t a, uint64_t b);

/* Bit operations */
extern int __clzsi2(uint32_t a);
extern int __popcountsi2(uint32_t a);

/* RISC-V register save/restore stubs */
extern void __riscv_save_0(void);
extern void __riscv_save_1(void);
extern void __riscv_save_2(void);
extern void __riscv_save_3(void);
extern void __riscv_save_4(void);
extern void __riscv_save_5(void);
extern void __riscv_save_6(void);
extern void __riscv_save_7(void);
extern void __riscv_save_8(void);
extern void __riscv_save_9(void);
extern void __riscv_save_10(void);
extern void __riscv_save_11(void);
extern void __riscv_save_12(void);

extern void __riscv_restore_0(void);
extern void __riscv_restore_1(void);
extern void __riscv_restore_2(void);
extern void __riscv_restore_3(void);
extern void __riscv_restore_4(void);
extern void __riscv_restore_5(void);
extern void __riscv_restore_6(void);
extern void __riscv_restore_7(void);
extern void __riscv_restore_8(void);
extern void __riscv_restore_9(void);
extern void __riscv_restore_10(void);
extern void __riscv_restore_11(void);
extern void __riscv_restore_12(void);

#ifdef __cplusplus
}
#endif

#endif /* GCC_BUILTINS_H */
