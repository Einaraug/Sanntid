package main

import (
	"fmt"
	"net"
	"os"
	"os/exec"
	"strconv"
	"time"
)

const (
	port              = ":20021"
	heartbeatInterval = 500 * time.Millisecond
	timeout           = 2 * time.Second
)

func main() {
	fmt.Println("=== PROGRAM STARTED ===")
	// Try to become the backup (listen for heartbeats from a primary)
	n := waitForPrimaryToDie()

	// If we reach here, we are now the primary
	fmt.Printf("=== BECOMING PRIMARY (starting from %d) ===\n", n)

	// Spawn a new backup before starting work
	spawnBackup()
	time.Sleep(time.Second)

	// Do the primary work: count and broadcast
	runAsPrimary(n)
}

// waitForPrimaryToDie listens for UDP heartbeats.
// Returns the last received count when the primary dies (or 0 if no primary exists).
func waitForPrimaryToDie() int {
	addr, err := net.ResolveUDPAddr("udp", port)
	if err != nil {
		fmt.Println("Error resolving address:", err)
		return 0
	}

	conn, err := net.ListenUDP("udp", addr)
	if err != nil {
		// Could not bind - likely another backup already listening
		// or we are the first process. Wait a bit and retry.
		fmt.Println("Could not bind to port (another backup may exist). Waiting...")
		time.Sleep(3 * time.Second)
		return waitForPrimaryToDie()
	}

	fmt.Println("=== RUNNING AS BACKUP (listening for primary) ===")

	buf := make([]byte, 1024)
	lastN := 0

	for {
		// Set read deadline for timeout detection
		conn.SetReadDeadline(time.Now().Add(timeout))

		n, _, err := conn.ReadFromUDP(buf)
		if err != nil {
			// Timeout or error - primary is dead
			if netErr, ok := err.(net.Error); ok && netErr.Timeout() {
				fmt.Println("Primary timed out! Taking over...")
				conn.Close() // Close BEFORE returning so new backup can use the port
				return lastN
			}
			// Some other error, assume primary is dead
			fmt.Println("Read error:", err)
			conn.Close()
			return lastN
		}

		// Parse the received count
		received, err := strconv.Atoi(string(buf[:n]))
		if err != nil {
			continue
		}
		lastN = received
		fmt.Printf("  [backup] Received heartbeat: %d\n", lastN)
	}
}

// runAsPrimary counts up from n and broadcasts heartbeats via UDP.
func runAsPrimary(n int) {
	// Create UDP connection for broadcasting
	addr, err := net.ResolveUDPAddr("udp", "127.0.0.1"+port)
	if err != nil {
		fmt.Println("Error resolving broadcast address:", err)
		return
	}

	conn, err := net.DialUDP("udp", nil, addr)
	if err != nil {
		fmt.Println("Error creating UDP connection:", err)
		return
	}
	defer conn.Close()

	// Count forever, broadcasting each number
	for {
		n++
		fmt.Printf("[PRIMARY] Count: %d\n", n)

		// Broadcast the current count to backup
		conn.Write([]byte(strconv.Itoa(n)))

		time.Sleep(heartbeatInterval)
	}
}

// spawnBackup starts a new instance of this program in a new terminal window.
func spawnBackup() {
	fmt.Println("Spawning backup process...")

	// Get current working directory
	dir, err := os.Getwd()
	if err != nil {
		fmt.Println("Error getting working directory:", err)
		return
	}

	// Source file - adjust this to match your filename
	sourceFile := dir + "/opus.go"
	fmt.Println("Spawning:", sourceFile)

	var cmd *exec.Cmd

	cmd = exec.Command("gnome-terminal", "--", "bash", "-c", "go run "+sourceFile+"; exec bash")

	err = cmd.Start()
	if err != nil {
		fmt.Println("Error spawning backup:", err)
		return
	}

	fmt.Println("Backup spawned successfully!")
}