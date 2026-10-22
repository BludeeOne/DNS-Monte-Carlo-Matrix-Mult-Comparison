CC      = gcc
CFLAGS  = -O2 -Wall -Wextra
LIBS    = -lpthread -lm
TARGET  = montecarlo
SRC     = montecarlo.c

all: $(TARGET)

$(TARGET): $(SRC)
	$(CC) $(CFLAGS) -o $(TARGET) $(SRC) $(LIBS)

clean:
	rm -f $(TARGET)

run-small: $(TARGET)
	./$(TARGET) 1000000 4

run-medium: $(TARGET)
	./$(TARGET) 10000000 4

run-large: $(TARGET)
	./$(TARGET) 100000000 4

run-all: $(TARGET)
	./$(TARGET) 1000000 4
	./$(TARGET) 10000000 4
	./$(TARGET) 100000000 4

.PHONY: all clean run-small run-medium run-large run-all