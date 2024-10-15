document.getElementById('registrationForm').addEventListener('submit', function(event) {
            event.preventDefault();

            const firstName = document.getElementById('firstName').value;
            const lastName = document.getElementById('lastName').value;
            const password = document.getElementById('password').value;

            sessionStorage.setItem('firstName', firstName);
            sessionStorage.setItem('lastName', lastName);
            sessionStorage.setItem('password', password);

            window.location.href = '/home/home.html';
        });
