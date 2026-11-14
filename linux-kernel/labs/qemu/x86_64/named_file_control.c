/* Independent pinned-Linux observations. This is never kernel-linked. */
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
#define CHECK(v) do { if (!(v)) {fprintf(stderr,"named-control:%d: %s errno=%d\n",__LINE__,#v,errno);exit(1);} } while(0)
static int first=1;
static void result(const char *label,int value) {
    printf("%s\"%s\":%d",first?"":",",label,value);first=0;
}
static void open_result(int directory,const char *name,int flags,const char *label) {
    errno=0;int fd=openat(directory,name,flags,0666);int error=fd<0?errno:0;
    if(fd>=0)CHECK(close(fd)==0);
    result(label,error);
}
static void unlink_result(int directory,const char *name,int flags,const char *label) {
    errno=0;int value=unlinkat(directory,name,flags);result(label,value<0?errno:0);
}
static int create(int directory,const char *name) {
    int fd=openat(directory,name,O_CREAT|O_EXCL|O_RDWR,0666);CHECK(fd>=0);return fd;
}
static struct stat status(int fd) {struct stat s;CHECK(fstat(fd,&s)==0);return s;}
static void written(int fd,const char *bytes,size_t n) {CHECK(write(fd,bytes,n)==(ssize_t)n);}
static void read_exact(int fd,char *bytes,size_t n) {CHECK(read(fd,bytes,n)==(ssize_t)n);}
static void seek_to(int fd,off_t at) {CHECK(lseek(fd,at,SEEK_SET)==at);}
static void lifetimes(int directory) {
    int a=create(directory,"a");written(a,"abcdefgh",8);
    int b=openat(directory,"a",O_RDONLY);CHECK(b>=0);
    int alias=dup(b);CHECK(alias>=0);char bytes[32]={0};
    read_exact(b,bytes,2);CHECK(memcmp(bytes,"ab",2)==0);
    read_exact(alias,bytes,2);CHECK(memcmp(bytes,"cd",2)==0);
    CHECK(lseek(a,0,SEEK_CUR)==8 && lseek(b,0,SEEK_CUR)==4);
    ino_t old=status(a).st_ino;
    int truncated=openat(directory,"a",O_RDONLY|O_TRUNC);CHECK(truncated>=0);
    CHECK(status(a).st_size==0 && status(a).st_blocks==0);
    CHECK((status(truncated).st_mode&0777)==0644);
    errno=0;CHECK(write(truncated,"x",1)==-1 && errno==EBADF);
    CHECK(lseek(a,0,SEEK_CUR)==8 && lseek(b,0,SEEK_CUR)==4);
    written(a,"x",1);seek_to(truncated,0);read_exact(truncated,bytes,9);
    CHECK(memcmp(bytes,"\0\0\0\0\0\0\0\0x",9)==0);
    CHECK(unlinkat(directory,"a",0)==0 && status(a).st_nlink==0);
    int replacement=create(directory,"a");CHECK(status(replacement).st_ino!=old);
    CHECK(status(replacement).st_size==0 && status(replacement).st_nlink==1);
    written(replacement,"new",3);seek_to(truncated,0);read_exact(truncated,bytes,9);
    CHECK(memcmp(bytes,"\0\0\0\0\0\0\0\0x",9)==0);
    CHECK(close(a)==0 && close(b)==0 && close(alias)==0 && close(truncated)==0 && close(replacement)==0);
    replacement=openat(directory,"a",O_RDONLY);CHECK(replacement>=0);
    read_exact(replacement,bytes,3);CHECK(memcmp(bytes,"new",3)==0);
    CHECK(close(replacement)==0 && unlinkat(directory,"a",0)==0);
}
struct native_dirent {uint64_t ino;int64_t offset;uint16_t length;unsigned char type;char name[];};
static void long_name(int directory) {
    char name[256];memset(name,0xff,255);name[10]=(char)0x80;name[255]=0;
    int fd=create(directory,name);CHECK(close(fd)==0);
    int view=openat(directory,".",O_RDONLY|O_DIRECTORY);CHECK(view>=0);
    char buffer[512]={0};
    CHECK(syscall(SYS_getdents64,view,buffer,48)==48);
    off_t cursor=lseek(view,0,SEEK_CUR);CHECK(cursor>=0);
    errno=0;CHECK(syscall(SYS_getdents64,view,buffer,279)==-1 && errno==EINVAL);
    CHECK(lseek(view,0,SEEK_CUR)==cursor);
    CHECK(syscall(SYS_getdents64,view,buffer,280)==280);
    struct native_dirent *entry=(void *)buffer;
    CHECK(entry->length==280 && entry->type==8 && memcmp(entry->name,name,256)==0);
    CHECK(syscall(SYS_getdents64,view,buffer,512)==0);
    CHECK(lseek(view,0,SEEK_SET)==0 && syscall(SYS_getdents64,view,buffer,48)==48);
    CHECK(close(view)==0 && unlinkat(directory,name,0)==0);
}
static void capacity(int directory) {
    int a=create(directory,"f0");written(a,"owned bytes",11);
    int other=openat(directory,"f0",O_RDONLY),alias=dup(a);CHECK(other>=0 && alias>=0);
    for(unsigned i=1;i<4;i++){char name[8];snprintf(name,sizeof(name),"f%u",i);int f=create(directory,name);CHECK(close(f)==0);}
    CHECK(unlinkat(directory,"f0",0)==0);
    errno=0;CHECK(openat(directory,"f4",O_CREAT|O_RDWR,0600)==-1 && errno==ENOSPC);
    CHECK(close(a)==0 && close(alias)==0);
    errno=0;CHECK(openat(directory,"f4",O_CREAT|O_RDWR,0600)==-1 && errno==ENOSPC);
    char bytes[16]={0};read_exact(other,bytes,11);CHECK(memcmp(bytes,"owned bytes",11)==0);
    CHECK(close(other)==0);int f=create(directory,"f4");CHECK(status(f).st_size==0 && status(f).st_blocks==0);
    CHECK(close(f)==0);
    for(unsigned i=1;i<=4;i++){char name[8];snprintf(name,sizeof(name),"f%u",i);CHECK(unlinkat(directory,name,0)==0);}
}
int main(int argc,char **argv) {
    CHECK(argc==3);umask(0022);
    int mutable=open(argv[1],O_RDONLY|O_DIRECTORY),immutable=open(argv[2],O_RDONLY|O_DIRECTORY);
    CHECK(mutable>=0 && immutable>=0);
    int file=create(mutable,"exists");CHECK(close(file)==0);
    printf("NAMED_FILE_CONTROL_PASS {");
    open_result(mutable,"missing/",O_CREAT|O_RDWR,"create_missing_trailing");
    open_result(mutable,"exists/",O_CREAT|O_RDWR,"create_regular_trailing");
    open_result(mutable,"exists/",O_CREAT|O_EXCL|O_RDWR,"exclusive_regular_trailing");
    open_result(mutable,"missing/",O_RDONLY,"read_missing_trailing");
    open_result(mutable,"exists",O_CREAT|O_EXCL|O_RDONLY,"exclusive_existing");
    open_result(mutable,".",O_CREAT|O_RDONLY,"create_dot");
    open_result(mutable,".",O_CREAT|O_EXCL|O_RDONLY,"exclusive_dot");
    unlink_result(mutable,"missing/",0,"unlink_missing_trailing");
    unlink_result(mutable,"exists/",0,"unlink_regular_trailing");
    unlink_result(mutable,".",0,"unlink_dot");
    unlink_result(mutable,"..",0,"unlink_parent");
    unlink_result(mutable,NULL,1,"unlink_invalid_flags_before_copy");
    open_result(immutable,"missing",O_CREAT|O_RDWR,"readonly_create_missing");
    open_result(immutable,"exists",O_CREAT|O_RDONLY,"readonly_create_existing");
    open_result(immutable,"exists",O_CREAT|O_EXCL|O_RDONLY,"readonly_exclusive_existing");
    open_result(immutable,".",O_CREAT|O_RDONLY,"readonly_create_dot");
    unlink_result(immutable,"missing",0,"readonly_unlink_missing");
    unlink_result(immutable,"missing/",0,"readonly_unlink_missing_trailing");
    char long_name_bytes[257];memset(long_name_bytes,'x',256);long_name_bytes[256]=0;
    unlink_result(immutable,long_name_bytes,0,"readonly_unlink_long_final");
    CHECK(unlinkat(mutable,"exists",0)==0);
    lifetimes(mutable);long_name(mutable);capacity(mutable);
    printf("}\n");CHECK(close(mutable)==0 && close(immutable)==0);return 0;
}
