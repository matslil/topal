/* Independent native layout probe; never linked into a kernel. */
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
struct history { void *creations; _Bool wrapped; uint32_t order[2]; };
struct state { struct history history; void *next; uint32_t order[2]; };
struct outcome { void *allocated; struct state state; uint32_t order[2]; };
struct number { uint64_t sign, length; uint32_t limbs[2]; };
int main(void) {
    const size_t facts[]={
        sizeof(struct history),sizeof(struct state),sizeof(struct outcome),sizeof(struct number),
        offsetof(struct history,creations),offsetof(struct history,wrapped),
        offsetof(struct history,order[0]),offsetof(struct history,order[1]),
        offsetof(struct state,next),offsetof(struct state,order[0]),offsetof(struct state,order[1]),
        offsetof(struct outcome,state),offsetof(struct outcome,order[0]),offsetof(struct outcome,order[1])
    };
    for(size_t i=0;i<sizeof(facts)/sizeof(facts[0]);i++)printf("%zu\n",facts[i]);
    return 0;
}
