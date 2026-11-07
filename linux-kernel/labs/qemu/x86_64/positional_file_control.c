/* Independent pinned-Linux observations; never kernel-linked. */
#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <unistd.h>
#define CHECK(v) do { if (!(v)) {fprintf(stderr,"positional-control:%d: %s errno=%d\n",__LINE__,#v,errno);exit(1);} } while(0)
static int first=1;
static long transfer(int writing,int fd,void *buffer,size_t count,int64_t position) {
    return syscall(writing?SYS_pwrite64:SYS_pread64,fd,buffer,count,position);
}
static void result(const char *label,int writing,int fd,void *buffer,size_t count,int64_t position) {
    errno=0;long value=transfer(writing,fd,buffer,count,position);
    printf("%s\"%s\":%d",first?"":",",label,value<0?errno:0);first=0;
}
static off_t position(int fd) {off_t value=lseek(fd,0,SEEK_CUR);CHECK(value>=0);return value;}
int main(int argc,char **argv) {
    CHECK(argc==3);
    int directory=open(argv[1],O_RDONLY|O_DIRECTORY);CHECK(directory>=0);
    int file=openat(directory,"position",O_CREAT|O_EXCL|O_RDWR,0600);CHECK(file>=0);
    CHECK(write(file,"abcdefgh",8)==8);
    int alias=dup(file),other=openat(directory,"position",O_RDWR),read_only=openat(directory,"position",O_RDONLY);
    int write_only=openat(directory,"position",O_WRONLY),append=openat(directory,"position",O_WRONLY|O_APPEND);
    int pipe_fds[2];CHECK(alias>=0 && other>=0 && read_only>=0 && write_only>=0 && append>=0 && pipe(pipe_fds)==0);
    int immutable_directory=open(argv[2],O_RDONLY|O_DIRECTORY);CHECK(immutable_directory>=0);
    int immutable=openat(immutable_directory,"exists",O_RDONLY);CHECK(immutable>=0);
    CHECK(lseek(other,2,SEEK_SET)==2);
    char bytes[32]={0};void *bad=(void *)(uintptr_t)UINT64_MAX;
    printf("POSITIONAL_FILE_CONTROL_PASS {");
    result("negative_bad_descriptor",0,-1,bad,0,-1);
    result("negative_write_bad_descriptor",1,-1,bad,0,-1);
    result("bad_descriptor",0,-1,bad,0,0);
    result("pipe_read_zero",0,pipe_fds[0],bad,0,0);
    result("pipe_write_zero",1,pipe_fds[1],bad,0,0);
    result("pipe_read_bad_buffer",0,pipe_fds[0],bad,1,0);
    result("pipe_wrong_direction",1,pipe_fds[0],bad,1,0);
    result("pipe_negative",0,pipe_fds[0],bad,1,-1);
    result("read_write_only",0,write_only,bad,0,0);
    result("write_read_only",1,read_only,bad,0,0);
    result("read_directory_zero",0,directory,bytes,0,0);
    result("read_directory_bad_buffer",0,directory,bad,1,0);
    result("read_directory_overflow",0,directory,bytes,1,INT64_MAX);
    result("write_directory",1,directory,bad,0,0);
    result("read_bad_buffer_zero",0,file,bad,0,0);
    result("write_bad_buffer_zero",1,file,bad,0,0);
    result("read_null_zero",0,file,NULL,0,0);
    result("write_null_zero",1,file,NULL,0,0);
    result("read_overflow",0,file,bytes,1,INT64_MAX);
    result("write_overflow",1,file,bytes,1,INT64_MAX);
    result("read_zero_max",0,file,bytes,0,INT64_MAX);
    result("write_zero_max",1,file,bytes,0,INT64_MAX);
    result("append_requested_overflow",1,append,bytes,2,INT64_MAX-1);
    result("append_zero_max",1,append,bytes,0,INT64_MAX);
    CHECK(transfer(0,immutable,bytes,8,2)==8 && memcmp(bytes,"mutable-",8)==0 && position(immutable)==0);
    CHECK(position(file)==8 && position(alias)==8 && position(other)==2 && position(append)==0);
    CHECK(transfer(0,file,bytes,3,2)==3 && memcmp(bytes,"cde",3)==0);
    CHECK(transfer(1,alias,"XY",2,1)==2);
    CHECK(transfer(0,other,bytes,8,0)==8 && memcmp(bytes,"aXYdefgh",8)==0);
    CHECK(transfer(1,append,"APP",3,0)==3);
    CHECK(transfer(0,file,bytes,11,0)==11 && memcmp(bytes,"aXYdefghAPP",11)==0);
    CHECK(position(file)==8 && position(alias)==8 && position(other)==2 && position(append)==0);
    CHECK(unlinkat(directory,"position",0)==0);
    struct stat status;CHECK(fstat(file,&status)==0 && status.st_nlink==0 && status.st_size==11);
    int64_t sparse=(INT64_C(1)<<48)+13;
    CHECK(transfer(1,other,"AB",2,sparse)==2);
    memset(bytes,0x5a,sizeof(bytes));CHECK(transfer(0,alias,bytes,8,sparse-6)==8);
    CHECK(memcmp(bytes,"\0\0\0\0\0\0AB",8)==0);
    CHECK(fstat(file,&status)==0 && status.st_size==sparse+2 && status.st_blocks==16);
    CHECK(position(file)==8 && position(alias)==8 && position(other)==2 && position(append)==0);
    for(unsigned i=0;i<2;i++)CHECK(close(pipe_fds[i])==0);
    CHECK(close(alias)==0 && close(other)==0 && close(read_only)==0 && close(write_only)==0 && close(append)==0 && close(file)==0 && close(directory)==0 && close(immutable)==0 && close(immutable_directory)==0);
    printf("}\n");return 0;
}
