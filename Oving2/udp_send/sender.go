package main

import (
	"net"
)

func main() {
	addr := net.UDPAddr{
		IP:   net.ParseIP("255.255.255.255"),  //Send til alle på lokalnettet
		Port: 30000,
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

	message := []byte("Hei") //Meldingen vi vil sende

	conn.WriteToUDP(message, &addr) //sender meldingen
}
