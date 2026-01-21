package main

import (
	"fmt"
	"net"
)

func main() {

	addr := net.UDPAddr{
		IP:   net.IPv4zero, 	// 0.0.0.0 Lytt på alle nettverksgrensesnitt
		Port: 20021, // porten som lyttes på
	}

	// Opprett UDP-"socket"
	conn, err := net.ListenUDP("udp", &addr)
	if err != nil {
		panic(err)
	}
	defer conn.Close()

	// Buffer for innkommende data
	buffer := make([]byte, 1024)

	fmt.Println("Lytter etter UDP-pakker på port", addr.Port)

	for {
		// Motta data
		n, fromWho, err := conn.ReadFromUDP(buffer)
		if err != nil {
			fmt.Println("Feil ved mottak:", err)
			continue
		}

		// Konverter bytes → string
		msg := string(buffer[:n])

		// (Valgfritt) filtrer bort meldinger fra deg selv
		// if !fromWho.IP.Equal(localIP) {
		fmt.Printf("Melding fra %s: %s\n", fromWho.String(), msg)
		// }
	}
}
