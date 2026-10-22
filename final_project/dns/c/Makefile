CC      = gcc
CFLAGS  = -O2 -Wall -Wextra
LIBS    = -lpthread
TARGET  = dns
SRCS    = dns.c queue.c util.c
OBJS    = $(SRCS:.c=.o)

all: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(CFLAGS) -o $(TARGET) $(OBJS) $(LIBS)

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(TARGET) $(OBJS) out_*.txt

run-small: $(TARGET)
	./$(TARGET) 50 4 out_small.txt

run-medium: $(TARGET)
	./$(TARGET) 200 4 out_medium.txt

run-large: $(TARGET)
	./$(TARGET) 500 4 out_large.txt

run-all: $(TARGET)
	./$(TARGET) 50  4 out_small.txt
	./$(TARGET) 200 4 out_medium.txt
	./$(TARGET) 500 4 out_large.txt

.PHONY: all clean run-small run-medium run-large run-all
