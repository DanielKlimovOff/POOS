document.getElementById('registrationForm').addEventListener('submit', async function(event) {
    event.preventDefault();

    const firstName = document.getElementById('firstName').value;
    const password = document.getElementById('password').value;

    try {
        const response = await fetch("http://localhost:3030/register", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                firstName: firstName,
                password: password
            }),
        });

        if (!response.ok) {
            throw new Error(`Server error: ${response.status}`);
        }

        const data = await response.json();
        console.log('Registration successful:', data);

        // Если регистрация прошла успешно, сохраняем имя пользователя и пароль
        sessionStorage.setItem('firstName', firstName);
        sessionStorage.setItem('password', password);

        // Перенаправляем пользователя на главную страницу
        window.location.href = '/home/home.html';
    } catch (error) {
        console.error('Error during registration:', error);
        alert('Failed to register. Please try again.');
    }
});
