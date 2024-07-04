FROM python:3

# Set the working directory inside the container
WORKDIR /papi_line

# Copy the requirements.txt file into the container's working directory
COPY requirements.txt ./

# Install the Python dependencies
# Run it before copying the rest of the project so that if the requirements.txt file does not change, the packages are not reinstalled
RUN pip install --no-cache-dir -r requirements.txt

# Copy the entire project into the /papi_line folder inside the container
# Exclude the files and folders specified in .dockerignore
COPY . .

# Expose the port which the HTTP server listens on
EXPOSE 6969

# Run the pAPI pipeline server
CMD [ "python", "./papi_line_server.py" ]