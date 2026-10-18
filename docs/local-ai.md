# VectorCraft AI Studio · RoboDhan

Buka RoboDhan integrasi terlebih dahulu. API lokal harus aktif di 127.0.0.1:1234. Model chat mengikuti pilihan aktif di RoboDhan; model FLUX bukan model chat.

Tombol AI di sidebar menampilkan atau menyembunyikan AI Studio. Masukkan deskripsi lalu klik Buat SVG. Hasil yang valid dibuka sebagai tab baru dengan objek vektor yang dapat diedit. Simpan melalui Save As.

Untuk mengubah ilustrasi, aktifkan Edit ilustrasi aktif. SVG dari dokumen aktif dikirim ke RoboDhan lokal bersama instruksi. Hasil tetap menjadi tab baru sehingga sumber tidak tertimpa. Dokumen yang memuat elemen di luar subset SVG sederhana akan ditolak dengan penjelasan.

SVG dibatasi 512 KiB, 5000 simpul dan kedalaman 32. Skrip, gambar tertanam, tautan eksternal, metadata dan SVG tidak lengkap ditolak. Kualitas ilustrasi mengikuti kemampuan model chat. Jika respons terpotong, gunakan prompt yang lebih sederhana.

Panel yang disembunyikan tetap menerima hasil. Abaikan hasil membuang respons setelah selesai; proses di RoboDhan dapat tetap berjalan. Tidak ada proses Python atau bridge tambahan dalam VectorCraft.
