package main

import (
	"fmt"
	"net"
)

func main() {

	addr := net.TCPAddr{
		IP: net.ParseIP("10.100.23.11"),
		Port: 33546,
	}

	conn, err := net.DialTCP("tcp", nil, &addr)
	if err != nil {
		panic(err)
	}
	defer conn.Close()
		// Les velkomstmelding
	buf := make([]byte, 1024)
	n, err := conn.Read(buf)
	if err != nil {
		panic(err)
	}

	fmt.Println(string(buf[:n]))


	message := []byte("Hei Matheus\x00") //Meldingen vi vil sende

	conn.Write(message) //sender meldingen

	buf = make([]byte, 1024)
	n, err = conn.Read(buf)
	if err != nil {
		panic(err)
	}

	fmt.Println(string(buf[:n]))

	listenaddr := net.TCPAddr{
		IP:   net.ParseIP("10.100.23.172"),
		Port: 20021,
	}

	// 3. Start å lytte (dette gjør deg til SERVER)
	listener, err := net.ListenTCP("tcp", &listenaddr)
	if err != nil {
		panic(err)
	}
	defer listener.Close()

	message = []byte("Connect to: 10.100.23.172:20021\000") //Meldingen vi vil sende

	conn.Write(message) //sender meldingen
	con, err := listener.AcceptTCP()
	if err != nil {
		panic(err)
	}
	defer con.Close()

	fmt.Println("Noen koblet seg til!")




	buf = make([]byte, 1024)
	n, err = con.Read(buf)
	if err != nil {
		panic(err)
	}
	fmt.Println("Mottatt:", string(buf[:n]))

	message = []byte("Hei Einar\000") //Meldingen vi vil sende

	con.Write(message) //sender meldingen

	buf = make([]byte, 1024)
	n, err = con.Read(buf)
	if err != nil {
		panic(err)
	}
	fmt.Println("Mottatt:", string(buf[:n]))
}