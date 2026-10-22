CC      = gcc
CFLAGS  = -O2 -Wall -Wextra
LIBS    = -lpthread -lm
TARGET  = matrix
SRC     = matrix.c

all: $(TARGET)

$(TARGET): $(SRC)
	$(CC) $(CFLAGS) -o $(TARGET) $(SRC) $(LIBS)

clean:
	rm -f $(TARGET)

run-small: $(TARGET)
	./$(TARGET) 256 4

run-medium: $(TARGET)
	./$(TARGET) 512 4

run-large: $(TARGET)
	./$(TARGET) 1024 4

run-all: $(TARGET)
	./$(TARGET) 256 4
	./$(TARGET) 512 4
	./$(TARGET) 1024 4

.PHONY: all clean run-small run-medium run-large run-all