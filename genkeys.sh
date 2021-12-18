openssl genrsa -out private.pem -des 2048
openssl req -new -x509 -key private.pem -out cert.pem -outform pem
openssl x509 -in cert.pem -inform pem -out public.der -outform der
rm cert.pem

