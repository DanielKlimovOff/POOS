document.getElementById('registrationForm').addEventListener('submit', function(event) {
            event.preventDefault();

            const firstName = document.getElementById('firstName').value;
            
            const password = document.getElementById('password').value;

            sessionStorage.setItem('firstName', firstName);
            sessionStorage.setItem('password', password);

            window.location.href = '/home/home.html';
        });
