import os
import sys
from cryptography.fernet import Fernet
import string
import secrets
from passlib.hash import argon2
import psycopg
from argon2 import PasswordHasher
from argon2.exceptions import HashingError
import logging
import random

# Generate a random password
def generate_random_password(length=12):
    characters = string.ascii_letters + string.digits + string.punctuation
    return ''.join(random.choice(characters) for i in range(length))

# Hash the password using Argon2
def hash_password(password):
    ph = PasswordHasher()
    try:
        return ph.hash(password)
    except HashingError as e:
        print(f"Error hashing password: {e}")
        return None

# Set up basic configuration for logging
logging.basicConfig(level=logging.ERROR, format='%(asctime)s - %(levelname)s - %(message)s')

# Append the pinepods directory to sys.path for module import
sys.path.append('/pinepods')

try:
    # Attempt to import additional modules
    import database_functions.functions
    # import Auth.Passfunctions

    def hash_password(password: str):
        # Hash the password
        hashed_password = argon2.hash(password)
        # Argon2 includes the salt in the hashed output
        return hashed_password



    # Database variables
    db_host = os.environ.get("DB_HOST", "127.0.0.1")
    db_port = os.environ.get("DB_PORT", "5432")
    db_user = os.environ.get("DB_USER", "postgres")
    db_password = os.environ.get("DB_PASSWORD", "password")
    db_name = os.environ.get("DB_NAME", "pypods_database")

    # Function to create the database if it doesn't exist
    def create_database_if_not_exists():
        try:
            # Connect to the default 'postgres' database
            with psycopg.connect(
                host=db_host,
                port=db_port,
                user=db_user,
                password=db_password,
                dbname='postgres'
            ) as conn:
                conn.autocommit = True
                with conn.cursor() as cur:
                    # Check if the database exists
                    cur.execute("SELECT 1 FROM pg_database WHERE datname = %s", (db_name,))
                    exists = cur.fetchone()
                    if not exists:
                        logging.info(f"Database {db_name} does not exist. Creating...")
                        print(f"Database {db_name} does not exist. Creating...")
                        cur.execute(f"CREATE DATABASE {db_name}")
                        logging.info(f"Database {db_name} created successfully.")
                    else:
                        logging.info(f"Database {db_name} already exists.")
        except Exception as e:
            logging.error(f"Error creating database: {e}")
            raise

    # Create the database if it doesn't exist
    create_database_if_not_exists()

    # Create database connector
    cnx = psycopg.connect(
        host=db_host,
        port=db_port,
        user=db_user,
        password=db_password,
        dbname=db_name
    )

    # create a cursor to execute SQL statements
    cursor = cnx.cursor()

    def ensure_usernames_lowercase(cnx):
        with cnx.cursor() as cursor:
            cursor.execute('SELECT UserID, Username FROM "Users"')
            users = cursor.fetchall()
            for user_id, username in users:
                if username != username.lower():
                    cursor.execute('UPDATE "Users" SET Username = %s WHERE UserID = %s', (username.lower(), user_id))
                    print(f"Updated Username for UserID {user_id} to lowercase")


    # Execute SQL command to create tables
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS "Users" (
            UserID SERIAL PRIMARY KEY,
            Fullname VARCHAR(255),
            Username VARCHAR(255) UNIQUE,
            Email VARCHAR(255),
            Hashed_PW VARCHAR(500),
            IsAdmin BOOLEAN,
            Reset_Code TEXT,
            Reset_Expiry TIMESTAMP,
            MFA_Secret VARCHAR(70),
            TimeZone VARCHAR(50) DEFAULT 'UTC',
            TimeFormat INT DEFAULT 24,
            DateFormat VARCHAR(3) DEFAULT 'ISO',
            FirstLogin BOOLEAN DEFAULT false,
            GpodderUrl VARCHAR(255) DEFAULT '',
            Pod_Sync_Type VARCHAR(50) DEFAULT 'None',
            GpodderLoginName VARCHAR(255) DEFAULT '',
            GpodderToken VARCHAR(255) DEFAULT '',
            EnableRSSFeeds BOOLEAN DEFAULT FALSE,
            PreferredLanguage VARCHAR(10) DEFAULT 'en'
        )
    """)

    # Add EnableRSSFeeds column if it doesn't exist
    cursor.execute("""
        SELECT column_name
        FROM information_schema.columns
        WHERE table_name = 'Users'
        AND column_name = 'enablerssfeeds'
    """)
    if cursor.fetchone() is None:
        cursor.execute('ALTER TABLE "Users" ADD COLUMN EnableRSSFeeds BOOLEAN DEFAULT FALSE')


    ensure_usernames_lowercase(cnx)


    cursor.execute("""CREATE TABLE IF NOT EXISTS "APIKeys" (
                        APIKeyID SERIAL PRIMARY KEY,
                        UserID INT,
                        APIKey TEXT,
                        Created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        FOREIGN KEY (UserID) REFERENCES "Users"(UserID) ON DELETE CASCADE
                    )""")

    cursor.execute("""CREATE TABLE IF NOT EXISTS "UserStats" (
                        UserStatsID SERIAL PRIMARY KEY,
                        UserID INT UNIQUE,
                        UserCreated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        PodcastsPlayed INT DEFAULT 0,
                        TimeListened INT DEFAULT 0,
                        PodcastsAdded INT DEFAULT 0,
                        EpisodesSaved INT DEFAULT 0,
                        EpisodesDownloaded INT DEFAULT 0,
                        FOREIGN KEY (UserID) REFERENCES "Users"(UserID)
                    )""")


    # Generate a key
    key = Fernet.generate_key()

    # Create the AppSettings table
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS "AppSettings" (
            AppSettingsID SERIAL PRIMARY KEY,
            SelfServiceUser BOOLEAN DEFAULT false,
            DownloadEnabled BOOLEAN DEFAULT true,
            EncryptionKey BYTEA,  -- Set the data type to BYTEA for binary data
            NewsFeedSubscribed BOOLEAN DEFAULT false
        )
    """)

    cursor.execute('SELECT COUNT(*) FROM "AppSettings" WHERE AppSettingsID = 1')
    count = cursor.fetchone()[0]

    if count == 0:
        cursor.execute("""
            INSERT INTO "AppSettings" (SelfServiceUser, DownloadEnabled, EncryptionKey)
            VALUES (false, true, %s)
        """, (key,))

    try:
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS "EmailSettings" (
                EmailSettingsID SERIAL PRIMARY KEY,
                Server_Name VARCHAR(255),
                Server_Port INT,
                From_Email VARCHAR(255),
                Send_Mode VARCHAR(255),
                Encryption VARCHAR(255),
                Auth_Required BOOLEAN,
                Username VARCHAR(255),
                Password VARCHAR(255)
            )
        """)
    except Exception as e:
        logging.error(f"Failed to create EmailSettings table: {e}")

    try:
        cursor.execute("""
            SELECT COUNT(*) FROM "EmailSettings"
        """)
        rows = cursor.fetchone()


        if rows[0] == 0:
            cursor.execute("""
                INSERT INTO "EmailSettings" (Server_Name, Server_Port, From_Email, Send_Mode, Encryption, Auth_Required, Username, Password)
                VALUES ('default_server', 587, 'default_email@domain.com', 'default_mode', 'default_encryption', true, 'default_username', 'default_password')
            """)
    except Exception as e:
        print(f"Error setting default email data: {e}")

    def user_exists(cursor, username):
        cursor.execute("""
            SELECT 1 FROM "Users" WHERE Username = %s
        """, (username,))
        return cursor.fetchone() is not None

    # Insert or update the user in the database
    def insert_or_update_user(cursor, hashed_password):
        try:
            if user_exists(cursor, 'guest'):
                cursor.execute("""
                    UPDATE "Users"
                    SET Fullname = %s, Username = %s, Email = %s, Hashed_PW = %s, IsAdmin = %s
                    WHERE Username = %s
                """, ('Background Tasks', 'background_tasks', 'inactive', hashed_password, False, 'guest'))
                logging.info("Updated existing 'guest' user to 'background_tasks' user.")
            elif user_exists(cursor, 'bt'):
                cursor.execute("""
                    UPDATE "Users"
                    SET Fullname = %s, Username = %s, Email = %s, Hashed_PW = %s, IsAdmin = %s
                    WHERE Username = %s
                """, ('Background Tasks', 'background_tasks', 'inactive', hashed_password, False, 'bt'))
                logging.info("Updated existing 'guest' user to 'background_tasks' user.")
            else:
                cursor.execute("""
                    INSERT INTO "Users" (Fullname, Username, Email, Hashed_PW, IsAdmin)
                    VALUES (%s, %s, %s, %s, %s)
                    ON CONFLICT (Username) DO NOTHING
                """, ('Background Tasks', 'background_tasks', 'inactive', hashed_password, False))
        except Exception as e:
            print(f"Error inserting or updating user: {e}")
            logging.error("Error inserting or updating user: %s", e)


    try:
        # Generate and hash the password
        random_password = generate_random_password()
        hashed_password = hash_password(random_password)

        if hashed_password:
            insert_or_update_user(cursor, hashed_password)



    except Exception as e:
        print(f"Error setting default Background Task User: {e}")
        logging.error("Error setting default Background Task User: %s", e)


    try:
        # Check if API key exists for user_id
        cursor.execute('SELECT apikey FROM "APIKeys" WHERE userid = %s', (1,))

        result = cursor.fetchone()

        if result:
            api_key = result[0]
        else:
            import secrets
            import string
            alphabet = string.ascii_letters + string.digits
            api_key = ''.join(secrets.choice(alphabet) for _ in range(64))

            # Insert the new API key into the database using a parameterized query
            cursor.execute('INSERT INTO "APIKeys" (UserID, APIKey) VALUES (%s, %s)', (1, api_key))

            cnx.commit()

        with open("/tmp/web_api_key.txt", "w") as f:
            f.write(api_key)
    except Exception as e:
        print(f"Error creating web key: {e}")

    try:
        cursor.execute("""CREATE TABLE IF NOT EXISTS "UserSettings" (
                            UserSettingID SERIAL PRIMARY KEY,
                            UserID INT UNIQUE,
                            Theme VARCHAR(255) DEFAULT 'Nordic',
                            FOREIGN KEY (UserID) REFERENCES "Users"(UserID)
                        )""")
    except Exception as e:
        print(f"Error adding UserSettings table: {e}")


    admin_created = False
    try:
        admin_fullname = os.environ.get("FULLNAME")
        admin_username = os.environ.get("USERNAME")
        admin_email = os.environ.get("EMAIL")
        admin_pw = os.environ.get("PASSWORD")

        if all([admin_fullname, admin_username, admin_email, admin_pw]):
            hashed_pw = hash_password(admin_pw).strip()
            admin_insert_query = """
                INSERT INTO "Users" (Fullname, Username, Email, Hashed_PW, IsAdmin)
                VALUES (%s, %s, %s, %s, %s::boolean)
                ON CONFLICT (Username) DO NOTHING
                RETURNING UserID
            """
            cursor.execute(admin_insert_query, (admin_fullname, admin_username, admin_email, hashed_pw, True))
            admin_created = cursor.fetchone() is not None
            cnx.commit()
    except Exception as e:
        print(f"Error creating default admin: {e}")

    # Now handle UserStats and UserSettings
    try:
        # Background tasks user stats
        cursor.execute("""
            INSERT INTO "UserStats" (UserID) VALUES (1)
            ON CONFLICT (UserID) DO NOTHING
        """)
        if admin_created:
            cursor.execute("""
                INSERT INTO "UserStats" (UserID) VALUES (2)
                ON CONFLICT (UserID) DO NOTHING
            """)
            cursor.execute("""
                INSERT INTO "UserSettings" (UserID, Theme) VALUES (2, 'Nordic')
                ON CONFLICT (UserID) DO NOTHING
            """)
        cnx.commit()
    except Exception as e:
        print(f"Error creating user stats/settings: {e}")

    cursor.execute("""INSERT INTO "UserSettings" (UserID, Theme) VALUES ('1', 'Nordic') ON CONFLICT (UserID) DO NOTHING""")
    if admin_created:
        cursor.execute("""INSERT INTO "UserSettings" (UserID, Theme) VALUES ('2', 'Nordic') ON CONFLICT (UserID) DO NOTHING""")


    try:
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS "Podcasts" (
                PodcastID SERIAL PRIMARY KEY,
                PodcastIndexID INT,
                PodcastName TEXT,
                ArtworkURL TEXT,
                Author TEXT,
                Categories TEXT,
                Description TEXT,
                EpisodeCount INT,
                FeedURL TEXT,
                WebsiteURL TEXT,
                Explicit BOOLEAN,
                UserID INT,
                AutoDownload BOOLEAN DEFAULT FALSE,
                StartSkip INT DEFAULT 0,
                EndSkip INT DEFAULT 0,
                Username TEXT,
                Password TEXT,
                IsYouTubeChannel BOOLEAN DEFAULT FALSE,
                FOREIGN KEY (UserID) REFERENCES "Users"(UserID)
            )
        """)
        cnx.commit()  # Ensure changes are committed
    except Exception as e:
        print(f"Error adding Podcasts table: {e}")

    def add_youtube_column_if_not_exist(cursor, cnx):
        try:
            cursor.execute("""
                SELECT column_name
                FROM information_schema.columns
                WHERE table_name='Podcasts'
                AND column_name = 'isyoutubechannel'
            """)
            existing_column = cursor.fetchone()

            if not existing_column:
                cursor.execute("""
                    ALTER TABLE "Podcasts"
                    ADD COLUMN "isyoutubechannel" BOOLEAN DEFAULT FALSE
                """)
                print("Added 'IsYouTubeChannel' column to 'Podcasts' table.")
                cnx.commit()
        except Exception as e:
            print(f"Error adding IsYouTubeChannel column to Podcasts table: {e}")

    # Usage
    add_youtube_column_if_not_exist(cursor, cnx)



    cursor.execute("SELECT to_regclass('public.\"Podcasts\"')")

    try:
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS "Episodes" (
                EpisodeID SERIAL PRIMARY KEY,
                PodcastID INT,
                EpisodeTitle TEXT,
                EpisodeDescription TEXT,
                EpisodeURL TEXT,
                EpisodeArtwork TEXT,
                EpisodePubDate TIMESTAMP,
                EpisodeDuration INT,
                Completed BOOLEAN DEFAULT FALSE,
                FOREIGN KEY (PodcastID) REFERENCES "Podcasts"(PodcastID)
            )
        """)

        cnx.commit()  # Ensure changes are committed
    except Exception as e:
        print(f"Error adding Episodes table: {e}")

    try:
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS "YouTubeVideos" (
                VideoID SERIAL PRIMARY KEY,
                PodcastID INT,
                VideoTitle TEXT,
                VideoDescription TEXT,
                VideoURL TEXT,
                ThumbnailURL TEXT,
                PublishedAt TIMESTAMP,
                Duration INT,
                YouTubeVideoID TEXT,
                Completed BOOLEAN DEFAULT FALSE,
                ListenPosition INT DEFAULT 0,
                FOREIGN KEY (PodcastID) REFERENCES "Podcasts"(PodcastID)
            )
        """)

        cnx.commit()  # Ensure changes are committed
    except Exception as e:
        print(f"Error adding YoutubeVideos table: {e}")

    def create_index_if_not_exists(cursor, index_name, table_name, column_name):
        cursor.execute(f"""
            SELECT 1
            FROM pg_indexes
            WHERE lower(indexname) = lower('{index_name}') AND tablename = '{table_name}'
        """)
        if not cursor.fetchone():
            cursor.execute(f'CREATE INDEX {index_name} ON "{table_name}"({column_name})')

    create_index_if_not_exists(cursor, "idx_podcasts_userid", "Podcasts", "UserID")
    create_index_if_not_exists(cursor, "idx_episodes_podcastid", "Episodes", "PodcastID")
    create_index_if_not_exists(cursor, "idx_episodes_episodepubdate", "Episodes", "EpisodePubDate")

    try:
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS "People" (
                PersonID SERIAL PRIMARY KEY,
                Name TEXT,
                PersonImg TEXT,
                PeopleDBID INT,
                AssociatedPodcasts TEXT,
                UserID INT,
                FOREIGN KEY (UserID) REFERENCES "Users"(UserID)
            );

        """)
        cnx.commit()
    except Exception as e:
        print(f"Error creating People table: {e}")

    try:
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS "PeopleEpisodes" (
                EpisodeID SERIAL PRIMARY KEY,
                PersonID INT,
                PodcastID INT,
                EpisodeTitle TEXT,
                EpisodeDescription TEXT,
                EpisodeURL TEXT,
                EpisodeArtwork TEXT,
                EpisodePubDate TIMESTAMP,
                EpisodeDuration INT,
                AddedDate TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (PersonID) REFERENCES "People"(PersonID),
                FOREIGN KEY (PodcastID) REFERENCES "Podcasts"(PodcastID)
            );

        """)
        cnx.commit()
    except Exception as e:
        print(f"Error creating People table: {e}")

    create_index_if_not_exists(cursor, "idx_people_episodes_person", "PeopleEpisodes", "PersonID")
    create_index_if_not_exists(cursor, "idx_people_episodes_podcast", "PeopleEpisodes", "PodcastID")


    try:
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS "SharedEpisodes" (
                SharedEpisodeID SERIAL PRIMARY KEY,
                EpisodeID INT,
                UrlKey TEXT,
                ExpirationDate TIMESTAMP,
                FOREIGN KEY (EpisodeID) REFERENCES "Episodes"(EpisodeID)
            )
        """)
        cnx.commit()
    except Exception as e:
        print(f"Error creating SharedEpisodes table: {e}")


    cursor.execute("""CREATE TABLE IF NOT EXISTS "UserEpisodeHistory" (
                        UserEpisodeHistoryID SERIAL PRIMARY KEY,
                        UserID INT,
                        EpisodeID INT,
                        ListenDate TIMESTAMP,
                        ListenDuration INT,
                        FOREIGN KEY (UserID) REFERENCES "Users"(UserID),
                        FOREIGN KEY (EpisodeID) REFERENCES "Episodes"(EpisodeID)
                    )""")

    cursor.execute("""
        CREATE TABLE IF NOT EXISTS "UserVideoHistory" (
            UserVideoHistoryID SERIAL PRIMARY KEY,
            UserID INT,
            VideoID INT,
            ListenDate TIMESTAMP,
            ListenDuration INT DEFAULT 0,
            FOREIGN KEY (UserID) REFERENCES "Users"(UserID),
            FOREIGN KEY (VideoID) REFERENCES "YouTubeVideos"(VideoID)
        )
    """)

    def add_history_constraints_if_not_exists(cursor, cnx):
        try:
            # Check/add constraint for UserEpisodeHistory
            cursor.execute("""
                SELECT constraint_name
                FROM information_schema.table_constraints
                WHERE table_name = 'UserEpisodeHistory'
                AND constraint_type = 'UNIQUE'
                AND constraint_name = 'user_episode_unique'
            """)

            if not cursor.fetchone():
                cursor.execute("""
                    ALTER TABLE "UserEpisodeHistory"
                    ADD CONSTRAINT user_episode_unique
                    UNIQUE (UserID, EpisodeID)
                """)
                print("Added unique constraint to UserEpisodeHistory table.")
                cnx.commit()

            # Check/add constraint for UserVideoHistory
            cursor.execute("""
                SELECT constraint_name
                FROM information_schema.table_constraints
                WHERE table_name = 'UserVideoHistory'
                AND constraint_type = 'UNIQUE'
                AND constraint_name = 'user_video_unique'
            """)

            if not cursor.fetchone():
                cursor.execute("""
                    ALTER TABLE "UserVideoHistory"
                    ADD CONSTRAINT user_video_unique
                    UNIQUE (UserID, VideoID)
                """)
                print("Added unique constraint to UserVideoHistory table.")
                cnx.commit()
        except Exception as e:
            print(f"Error adding unique constraints to history tables: {e}")

    # Call this after creating both history tables
    add_history_constraints_if_not_exists(cursor, cnx)

    cursor.execute("""CREATE TABLE IF NOT EXISTS "SavedEpisodes" (
                        SaveID SERIAL PRIMARY KEY,
                        UserID INT,
                        EpisodeID INT,
                        SaveDate TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        FOREIGN KEY (UserID) REFERENCES "Users"(UserID),
                        FOREIGN KEY (EpisodeID) REFERENCES "Episodes"(EpisodeID)
                    )""")

    cursor.execute("""CREATE TABLE IF NOT EXISTS "SavedVideos" (
        SaveID SERIAL PRIMARY KEY,
        UserID INT,
        VideoID INT,
        SaveDate TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (UserID) REFERENCES "Users"(UserID),
        FOREIGN KEY (VideoID) REFERENCES "YouTubeVideos"(VideoID)
        )""")


    # Create the DownloadedEpisodes table
    cursor.execute("""CREATE TABLE IF NOT EXISTS "DownloadedEpisodes" (
                    DownloadID SERIAL PRIMARY KEY,
                    UserID INT,
                    EpisodeID INT,
                    DownloadedDate TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    DownloadedSize INT,
                    DownloadedLocation VARCHAR(255),
                    FOREIGN KEY (UserID) REFERENCES "Users"(UserID),
                    FOREIGN KEY (EpisodeID) REFERENCES "Episodes"(EpisodeID)
                    )""")

    cursor.execute("""CREATE TABLE IF NOT EXISTS "DownloadedVideos" (
        DownloadID SERIAL PRIMARY KEY,
        UserID INT,
        VideoID INT,
        DownloadedDate TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        DownloadedSize INT,
        DownloadedLocation VARCHAR(255),
        FOREIGN KEY (UserID) REFERENCES "Users"(UserID),
        FOREIGN KEY (VideoID) REFERENCES "YouTubeVideos"(VideoID)
        )""")

    # Create the EpisodeQueue table
    cursor.execute("""CREATE TABLE IF NOT EXISTS "EpisodeQueue" (
        QueueID SERIAL PRIMARY KEY,
        QueueDate TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        UserID INT,
        EpisodeID INT,
        QueuePosition INT NOT NULL DEFAULT 0,
        is_youtube BOOLEAN DEFAULT FALSE,
        FOREIGN KEY (UserID) REFERENCES "Users"(UserID)
    )""")

    def remove_episode_queue_constraint(cursor, cnx):
        try:
            # First check if the constraint exists
            check_constraint_query = """
                SELECT constraint_name 
                FROM information_schema.table_constraints 
                WHERE table_name = 'EpisodeQueue' 
                AND constraint_name = 'EpisodeQueue_episodeid_fkey'
                AND constraint_type = 'FOREIGN KEY'
            """
            cursor.execute(check_constraint_query)
            constraint = cursor.fetchone()
            
            if constraint:
                # If it exists, drop it
                cursor.execute('ALTER TABLE "EpisodeQueue" DROP CONSTRAINT "EpisodeQueue_episodeid_fkey"')
                cnx.commit()
                print("Removed EpisodeQueue foreign key constraint")
            else:
                print("EpisodeQueue foreign key constraint not found - no action needed")
                
        except Exception as e:
            print(f"Error managing EpisodeQueue constraint: {e}")
            cnx.rollback()

    remove_episode_queue_constraint(cursor, cnx)

    def add_queue_youtube_column_if_not_exist(cursor, cnx):
        try:
            # Check if column exists using PostgreSQL's system catalog
            cursor.execute("""
                SELECT EXISTS (
                    SELECT 1 
                    FROM information_schema.columns 
                    WHERE table_name = 'EpisodeQueue' 
                    AND column_name = 'is_youtube'
                )
            """)
            column_exists = cursor.fetchone()[0]
            
            if not column_exists:
                cursor.execute("""
                    ALTER TABLE "EpisodeQueue"
                    ADD COLUMN is_youtube BOOLEAN DEFAULT FALSE
                """)
                cnx.commit()
                print("Added 'is_youtube' column to 'EpisodeQueue' table.")
            else:
                print("Column 'is_youtube' already exists in 'EpisodeQueue' table.")
                
        except Exception as e:
            cnx.rollback()
            print(f"Error managing is_youtube column: {e}")

    add_queue_youtube_column_if_not_exist(cursor, cnx)

    # Create the Sessions table
    cursor.execute("""CREATE TABLE IF NOT EXISTS "Sessions" (
                    SessionID SERIAL PRIMARY KEY,
                    UserID INT,
                    value TEXT,
                    expire TIMESTAMP NOT NULL,
                    FOREIGN KEY (UserID) REFERENCES "Users"(UserID)
                    )""")
    cnx.commit()


except psycopg.Error as err:
    logging.error(f"Database error: {err}")
except Exception as e:
    logging.error(f"General error: {e}")

# Ensure to close the cursor and connection
finally:
    if 'cursor' in locals():
        cursor.close()
    if 'cnx' in locals():
        cnx.close()
