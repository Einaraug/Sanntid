package main

import (
	"net"
)

func main() {
	addr := net.UDPAddr{
		IP:   net.ParseIP("10.100.23.11"),  //Send til alle på lokalnettet
		Port: 20019,
	}

	conn, err := net.ListenUDP("udp", nil) //Lager UDP socket
	if err != nil {
		panic(err)
	}
	defer conn.Close()

	err = conn.SetWriteBuffer(1024)
	if err != nil {
		panic(err)
	}

	message := []byte("Hei Matheus") //Meldingen vi vil sende

	conn.WriteToUDP(message, &addr) //sender meldingen
}
